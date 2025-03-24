#!cfg[(target_os = "uefi")]
//!
//! http network boot config utility
//!
//! usage (efi shell prompt):
//!   `netboot.efi [ $url | xyz ]`
//!
//! usage (qemu command line):
//!   `qemu -kernel netboot.efi -append $url`
//!
#![no_main]
#![no_std]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::proto::device_path::build;
use uefi::proto::device_path::DevicePath;
use uefi::runtime::VariableAttributes;
use uefi::runtime::VariableVendor;
use uefi::CString16;
use uefi::{print, println};
use uefi_raw::protocol::network::ip4_config2::Ip4Config2Policy;
use uefi_raw::Ipv4Address;

use efi_apps::distro::coreos;
use efi_apps::misc;
use efi_apps::net::http::{HttpBinding, HttpHelper};
use efi_apps::net::ip4config2::Ip4Config2;

fn init_nic_ip4config2() -> Option<Handle> {
    let Ok(handles) = boot::locate_handle_buffer(boot::SearchType::from_proto::<HttpBinding>())
    else {
        println!("no HttpBinding handles found");
        return None;
    };

    println!("found {} HttpBinding handle(s)", handles.as_ref().len());
    for h in handles.as_ref() {
        misc::print_handle_devpath("  nic: ", h);

        let ip4_res = Ip4Config2::new(*h);
        if let Err(e) = ip4_res {
            println!("  error: ip4cfg new: {}", e);
            continue;
        }
        let mut ip4 = ip4_res.unwrap();
        let noip = Ipv4Address::default();

        let info = ip4.get_interface_info().ok()?;
        if info.station_addr != noip {
            println!("  network is up, addr {:?}", info.station_addr);
            return Some(*h);
        }

        print!("  kick dhcp ");
        ip4.set_policy(Ip4Config2Policy::DHCP).ok()?;
        for _ in 0..30 {
            print!(".");
            boot::stall(1_000_000);
            let info = ip4.get_interface_info().ok()?;
            if info.station_addr != noip {
                println!(" got addr {:?}", info.station_addr);
                return Some(*h);
            }
        }
    }

    None
}

fn fetch_http(handle: Handle, url: &str) -> Option<Vec<u8>> {
    println!("  http fetch: {} ...", url);
    let http_res = HttpHelper::new(handle);
    if let Err(e) = http_res {
        println!("  error: http new: {}", e);
        return None;
    }
    let mut http = http_res.unwrap();

    let res = http.configure();
    if let Err(e) = res {
        println!("  error: http configure: {}", e);
        return None;
    }

    let res = http.request_get(url);
    if let Err(e) = res {
        println!("  error: http request: {}", e);
        return None;
    }

    let res = http.response_first(true);
    if let Err(e) = res {
        println!("  error: http response: {}", e);
        return None;
    }

    let rsp = res.unwrap();
    println!("    result: {:?}", rsp.status);

    let Some(cl_hdr) = rsp.headers.iter().find(|h| h.0 == "content-length") else {
        println!("  error: no content length");
        return None;
    };
    let Ok(cl) = cl_hdr.1.parse::<usize>() else {
        println!("  error: parse content length ({})", cl_hdr.1);
        return None;
    };
    println!("    content-length: {}", cl);

    let mut data = rsp.body;
    loop {
        println!("    body: {}/{}", data.len(), cl);
        if data.len() >= cl {
            break;
        }

        let res = http.response_more();
        if let Err(e) = res {
            println!("  error: http more data: {}", e);
            return None;
        }

        let mut buf = res.unwrap();
        data.append(&mut buf);
    }

    Some(data)
}

fn coreos(stream: &str) -> Option<String> {
    misc::connect_all(true);
    let Some(handle) = init_nic_ip4config2() else {
        println!("failed to configure network");
        return None;
    };

    let uri = format!("https://builds.coreos.fedoraproject.org/streams/{stream}.json");
    let data = fetch_http(handle, &uri);
    if data.is_none() {
        println!("failed fetch coreos stream");
        return None;
    }

    let core = serde_json::from_slice::<coreos::Stream>(&data.unwrap());
    if let Err(e) = core {
        println!("failed parse coreos stream: {}", e);
        return None;
    }

    let arch = core.unwrap().architectures.x86_64.artifacts.metal;
    println!("  coreos release: {}", arch.release);
    println!("  url: {}", arch.formats.iso.disk.location);

    println!("  ok, preparing for coreos boot ...");
    Some(arch.formats.iso.disk.location)
}

fn bootentry(title: &str, devpath: &DevicePath, optdata: Option<&[u8]>) -> Vec<u8> {
    let mut bootentry = Vec::new();
    let devpath_bytes = devpath.as_bytes();
    let title_cstr16 = CString16::try_from(title).unwrap();
    bootentry.extend(u32::to_le_bytes(1)); // attribute: active
    bootentry.extend(u16::to_le_bytes(devpath_bytes.len() as u16));
    bootentry.extend(title_cstr16.as_bytes());
    bootentry.extend(devpath_bytes);
    if let Some(o) = optdata {
        bootentry.extend(o);
    }
    bootentry
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let bootnext = CString16::try_from("BootNext").unwrap();
    if runtime::variable_exists(&bootnext, &VariableVendor::GLOBAL_VARIABLE).unwrap() {
        // we land here after reset when loaded via 'qemu -kernel netboot.efi'
        println!("netboot: BootNext exists, nothing to do.");
        boot::stall(1_000_000);
        return Status::SUCCESS;
    };

    let Some(cmdline) = misc::cmdline() else {
        println!("usage: netboot.efi $url");
        boot::stall(1_000_000);
        return Status::SUCCESS;
    };

    let uri = match cmdline.as_str() {
        "xyz" => "http://boot.netboot.xyz/ipxe/netboot.xyz.efi",
        "coreos" => {
            let iso = coreos("stable");
            if iso.is_none() {
                println!("failed to find latest coreos iso");
                boot::stall(1_000_000);
                return Status::SUCCESS;
            }
            &iso.unwrap()
        }
        _ => &cmdline,
    };

    println!("Creating BootEntry ...");
    println!("  booturi : {}", uri);

    let title = uri.split('/').last().unwrap();
    println!("  title   : {}", title);

    let mut vec = Vec::new();
    let devpath = build::DevicePathBuilder::with_vec(&mut vec)
        .push(&build::messaging::Uri {
            value: uri.as_bytes(),
        })
        .unwrap()
        .finalize()
        .unwrap();
    //println!("  devpath : {:?}", devpath);

    let battr = VariableAttributes::BOOTSERVICE_ACCESS
        | VariableAttributes::RUNTIME_ACCESS
        | VariableAttributes::NON_VOLATILE;
    let boot99 = CString16::try_from("Boot0099").unwrap();
    let bentry = bootentry(title, devpath, None);

    let rc = runtime::set_variable(&boot99, &VariableVendor::GLOBAL_VARIABLE, battr, &bentry);
    println!("Set Boot0099: {:?}", rc);
    if let Err(e) = rc {
        return e.status();
    }

    let rc = runtime::set_variable(
        &bootnext,
        &VariableVendor::GLOBAL_VARIABLE,
        battr,
        &u16::to_le_bytes(0x99),
    );
    println!("Set BootNext: {:?}", rc);
    if let Err(e) = rc {
        return e.status();
    }

    println!("");
    println!("Ready, rebooting in 3 sec.");
    boot::stall(3_000_000);
    runtime::reset(runtime::ResetType::COLD, Status::SUCCESS, None);
}
