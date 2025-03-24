#!cfg[(target_os = "uefi")]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::{c_void, CStr};
use core::ptr;
use log::debug;

use uefi::boot::ScopedProtocol;
use uefi::prelude::*;
use uefi::proto::unsafe_protocol;
use uefi_raw::protocol::driver::ServiceBindingProtocol;
use uefi_raw::protocol::network::http::{
    HttpAccessPoint, HttpConfigData, HttpHeader, HttpMessage, HttpMethod, HttpProtocol,
    HttpRequestData, HttpResponseData, HttpStatusCode, HttpToken, HttpV4AccessPoint, HttpVersion,
};

#[derive(Debug)]
#[unsafe_protocol(HttpProtocol::GUID)]
pub struct Http(HttpProtocol);

impl Http {
    pub fn configure(&mut self, config_data: &HttpConfigData) -> uefi::Result<()> {
        let status = unsafe { (self.0.configure)(&mut self.0, config_data) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    pub fn request(&mut self, token: &mut HttpToken) -> uefi::Result<()> {
        let status = unsafe { (self.0.request)(&mut self.0, token) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    pub fn response(&mut self, token: &mut HttpToken) -> uefi::Result<()> {
        let status = unsafe { (self.0.response)(&mut self.0, token) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    pub fn poll(&mut self) -> uefi::Result<()> {
        let status = unsafe { (self.0.poll)(&mut self.0) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }
}

#[derive(Debug)]
#[unsafe_protocol(HttpProtocol::SERVICE_BINDING_GUID)]
pub struct HttpBinding(ServiceBindingProtocol);

impl HttpBinding {
    pub fn create_child(&mut self) -> uefi::Result<Handle> {
        let mut c_handle = ptr::null_mut();
        let status;
        let handle;
        unsafe {
            status = (self.0.create_child)(&mut self.0, &mut c_handle);
            handle = Handle::from_ptr(c_handle);
        };
        match status {
            Status::SUCCESS => Ok(handle.unwrap()),
            _ => Err(status.into()),
        }
    }

    pub fn destroy_child(&mut self, handle: Handle) -> uefi::Result<()> {
        let status = unsafe { (self.0.destroy_child)(&mut self.0, handle.as_ptr()) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }
}

#[derive(Debug)]
pub struct HttpHelperResponse {
    pub status: HttpStatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct HttpHelper {
    child_handle: Option<Handle>,
    binding: Option<ScopedProtocol<HttpBinding>>,
    protocol: Option<ScopedProtocol<Http>>,
}

impl HttpHelper {
    pub fn new(nic_handle: Handle) -> uefi::Result<HttpHelper> {
        let mut helper = HttpHelper {
            child_handle: None,
            binding: None,
            protocol: None,
        };

        let mut binding;
        unsafe {
            binding = boot::open_protocol::<HttpBinding>(
                boot::OpenProtocolParams {
                    handle: nic_handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                boot::OpenProtocolAttributes::GetProtocol,
            )?;
        }
        debug!("http: binding proto ok");

        let child_handle = binding.create_child()?;
        helper.binding = Some(binding);
        helper.child_handle = Some(child_handle);
        debug!("http: child handle ok");

        let protocol;
        unsafe {
            protocol = boot::open_protocol::<Http>(
                boot::OpenProtocolParams {
                    handle: child_handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                boot::OpenProtocolAttributes::GetProtocol,
            )?;
        }
        helper.protocol = Some(protocol);
        debug!("http: protocol ok");

        Ok(helper)
    }

    pub fn configure(&mut self) -> uefi::Result<()> {
        let ip4 = HttpV4AccessPoint {
            use_default_addr: true,
            ..Default::default()
        };

        let config = HttpConfigData {
            http_version: HttpVersion::HTTP_VERSION_10,
            time_out_millisec: 10_000,
            local_addr_is_ipv6: false,
            access_point: HttpAccessPoint { ipv4_node: &ip4 },
        };

        let p = self.protocol.as_mut().expect("no http protocol");
        p.configure(&config)?;
        debug!("http: configure ok");

        Ok(())
    }

    pub fn request(
        &mut self,
        method: HttpMethod,
        url: &str,
        body: Option<&mut [u8]>,
    ) -> uefi::Result<()> {
        let url16 = uefi::CString16::try_from(url).unwrap();

        let Some(hostname) = url.split('/').nth(2) else {
            return Err(Status::INVALID_PARAMETER.into());
        };
        let mut c_hostname = String::from(hostname);
        c_hostname.push('\0');
        debug!("http: host: {}", hostname);

        let mut tx_req = HttpRequestData {
            method,
            url: url16.as_ptr() as *const u16,
        };

        let mut tx_hdr = Vec::new();
        tx_hdr.push(HttpHeader {
            field_name: c"Host".as_ptr() as *const u8,
            field_value: c_hostname.as_ptr(),
        });

        let mut tx_msg = HttpMessage::default();
        tx_msg.data.request = &mut tx_req;
        tx_msg.header_count = tx_hdr.len();
        tx_msg.header = tx_hdr.as_mut_ptr();
        if body.is_some() {
            let b = body.unwrap();
            tx_msg.body_length = b.len();
            tx_msg.body = b.as_mut_ptr() as *mut c_void;
        }

        let mut tx_token = HttpToken {
            status: Status::NOT_READY,
            message: &mut tx_msg,
            ..Default::default()
        };

        let p = self.protocol.as_mut().expect("no http protocol");
        p.request(&mut tx_token)?;
        debug!("http: request sent ok");

        loop {
            if tx_token.status != Status::NOT_READY {
                break;
            }
            p.poll()?;
        }

        if tx_token.status != Status::SUCCESS {
            return Err(tx_token.status.into());
        };

        debug!("http: request status ok");

        Ok(())
    }

    pub fn request_get(&mut self, url: &str) -> uefi::Result<()> {
        self.request(HttpMethod::GET, url, None)?;
        Ok(())
    }

    pub fn request_head(&mut self, url: &str) -> uefi::Result<()> {
        self.request(HttpMethod::HEAD, url, None)?;
        Ok(())
    }

    pub fn response(&mut self, body: &mut [u8]) -> uefi::Result<HttpHelperResponse> {
        let mut rx_rsp = HttpResponseData {
            status_code: HttpStatusCode::STATUS_UNSUPPORTED,
        };

        let mut rx_msg = HttpMessage::default();
        rx_msg.data.response = &mut rx_rsp;
        rx_msg.body_length = body.len();
        rx_msg.body = if !body.is_empty() {
            body.as_mut_ptr()
        } else {
            ptr::null()
        } as *mut c_void;

        let mut rx_token = HttpToken {
            status: Status::NOT_READY,
            message: &mut rx_msg,
            ..Default::default()
        };

        let p = self.protocol.as_mut().expect("no http protocol");
        p.response(&mut rx_token)?;

        loop {
            if rx_token.status != Status::NOT_READY {
                break;
            }
            p.poll()?;
        }

        debug!(
            "http: response: {} / {:?}",
            rx_token.status, rx_rsp.status_code
        );

        if rx_token.status != Status::SUCCESS && rx_token.status != Status::HTTP_ERROR {
            return Err(rx_token.status.into());
        };

        debug!("http: headers: {}", rx_msg.header_count);
        let mut headers: Vec<(String, String)> = Vec::new();
        for i in 0..rx_msg.header_count {
            let n;
            let v;
            unsafe {
                n = CStr::from_ptr((*rx_msg.header.add(i)).field_name as *const i8);
                v = CStr::from_ptr((*rx_msg.header.add(i)).field_value as *const i8);
            }
            headers.push((
                n.to_str().unwrap().to_lowercase(),
                String::from(v.to_str().unwrap()),
            ));
        }

        debug!("http: body: {}/{}", rx_msg.body_length, body.len());

        let rsp = HttpHelperResponse {
            status: rx_rsp.status_code,
            headers,
            body: body[0..rx_msg.body_length].to_vec(),
        };
        Ok(rsp)
    }
}

impl Drop for HttpHelper {
    fn drop(&mut self) {
        debug!("http: drop");
        self.protocol = None;
        if self.binding.is_some() {
            let b = self.binding.as_mut().unwrap();
            let _ = b.destroy_child(self.child_handle.unwrap());
        }
        self.binding = None;
    }
}
