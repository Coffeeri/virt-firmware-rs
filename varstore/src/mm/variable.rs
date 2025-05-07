//!
//! [WIP] EfiSmmVariableProtocol reimplementation
//!
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Display;
use log::{debug, error};
use uguid::Guid;
use zerocopy::{FromBytes, IntoBytes};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};

use uefi::{CStr16, Error, Status};
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};

use crate::mm::util;
use crate::store::EfiVarStore;

pub const GET_VARIABLE: u64 = 1;
pub const GET_NEXT_VARIABLE_NAME: u64 = 2;
pub const SET_VARIABLE: u64 = 3;
pub const QUERY_VARIABLE_INFO: u64 = 4;
pub const READY_TO_BOOT: u64 = 5;
pub const EXIT_BOOT_SERVICE: u64 = 6;
pub const LOCK_VARIABLE: u64 = 8;
pub const GET_PAYLOAD_SIZE: u64 = 11;

// SMM_VARIABLE_COMMUNICATE_HEADER
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable)]
pub struct MmVariableHeader {
    pub function: u64,
    pub status: u64,
}

impl MmVariableHeader {
    pub fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read MmVariableHeader",
        )))
    }
}

// SMM_VARIABLE_COMMUNICATE_ACCESS_VARIABLE
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable)]
pub struct MmVariableAccess {
    pub guid: [u8; 16],
    pub data_size: u64,
    pub name_size: u64,
    pub attributes: EfiVarAttr,
    // Name
    // Data
}

impl MmVariableAccess {
    pub fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read MmVariableAccess",
        )))
    }
}

// SMM_VARIABLE_COMMUNICATE_GET_NEXT_VARIABLE_NAME
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable)]
pub struct MmNextVariable {
    pub guid: [u8; 16],
    pub name_size: u64,
    // Name
}

impl MmNextVariable {
    pub fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read MmNextVariable",
        )))
    }
}

// SMM_VARIABLE_COMMUNICATE_QUERY_VARIABLE_INFO
#[repr(C, packed)]
#[derive(Debug, FromBytes)]
pub struct MmVariableInfo {
    pub max_storage_size: u64,
    pub free_storage_size: u64,
    pub max_variable_size: u64,
    pub attributes: u32,
}

// SMM_VARIABLE_COMMUNICATE_GET_PAYLOAD_SIZE
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable)]
pub struct MmGetPayloadSize {
    pub payload_size: u64,
}

// parsed request
#[derive(Debug)]
pub enum MmVariableParsedRequest<'a> {
    GetVariable {
        guid: Guid,
        name: &'a CStr16,
    },
    GetNextVariableName {
        guid: Guid,
        name: &'a CStr16,
    },
    SetVariable {
        guid: Guid,
        name: &'a CStr16,
        attr: EfiVarAttr,
        data: &'a [u8],
    },
    // QueryVariableInfo = 4,
    ReadyToBoot,
    ExitBootService,
    LockVariable {
        guid: Guid,
        name: &'a CStr16,
    },
    GetPayloadSize,
    Unknown {
        function: u64,
    },
}

impl<'a> MmVariableParsedRequest<'a> {
    pub fn new(function: u64, body: &'a [u8]) -> Result<Self, Error<&'static str>> {
        let req = match function {
            GET_VARIABLE => {
                let (access, namedata) = MmVariableAccess::new(body)?;
                let ns = access.name_size as usize;
                let name = util::cstr16_from_bytes_with_nul(&namedata[0..ns])?;
                let guid = Guid::from_bytes(access.guid);
                MmVariableParsedRequest::GetVariable { guid, name }
            }
            GET_NEXT_VARIABLE_NAME | LOCK_VARIABLE => {
                let (next, namebytes) = MmNextVariable::new(body)?;
                let ns = next.name_size as usize;
                let name = util::cstr16_from_bytes_until_nul(&namebytes[0..ns])?;
                let guid = Guid::from_bytes(next.guid);
                if function == GET_NEXT_VARIABLE_NAME {
                    MmVariableParsedRequest::GetNextVariableName { guid, name }
                } else {
                    MmVariableParsedRequest::LockVariable { guid, name }
                }
            }
            SET_VARIABLE => {
                let (access, namedata) = MmVariableAccess::new(body)?;
                let ns = access.name_size as usize;
                let ds = access.data_size as usize;
                let name = util::cstr16_from_bytes_with_nul(&namedata[0..ns])?;
                let data = &namedata[ns..ns + ds];
                let guid = Guid::from_bytes(access.guid);
                MmVariableParsedRequest::SetVariable {
                    guid,
                    name,
                    attr: access.attributes,
                    data,
                }
            }
            READY_TO_BOOT => MmVariableParsedRequest::ReadyToBoot,
            EXIT_BOOT_SERVICE => MmVariableParsedRequest::ExitBootService,
            GET_PAYLOAD_SIZE => MmVariableParsedRequest::GetPayloadSize,
            _ => MmVariableParsedRequest::Unknown { function },
        };
        Ok(req)
    }
}

impl Display for MmVariableParsedRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            MmVariableParsedRequest::GetVariable { guid: _, name } => {
                write!(f, "request/var: get-variable {name}")
            }
            MmVariableParsedRequest::GetNextVariableName { guid: _, name } => {
                write!(f, "request/var: get-next-variable-name {name}")
            }
            MmVariableParsedRequest::SetVariable {
                guid: _,
                name,
                attr,
                data,
            } => {
                write!(
                    f,
                    "request/var: set-variable {name}  {attr}  {} data bytes",
                    data.len()
                )
            }
            MmVariableParsedRequest::ReadyToBoot => {
                write!(f, "request/var: ready-to-boot")
            }
            MmVariableParsedRequest::ExitBootService => {
                write!(f, "request/var: exit-boot-service")
            }
            MmVariableParsedRequest::LockVariable { guid: _, name } => {
                write!(f, "request/var: lock-variable {name}")
            }
            MmVariableParsedRequest::GetPayloadSize => {
                write!(f, "request/var: get-payload-size")
            }
            MmVariableParsedRequest::Unknown { function } => {
                write!(f, "request/var: unknown function #{function}")
            }
        }
    }
}

// request state
#[derive(Debug)]
pub struct MmVariableRequest<'a> {
    pub data: &'a [u8],
    pub function: u64,
    pub parsed: Option<MmVariableParsedRequest<'a>>,
}

impl<'a> MmVariableRequest<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error<&'static str>> {
        let (header, body) = MmVariableHeader::new(data)?;
        match MmVariableParsedRequest::new(header.function, body) {
            Ok(req) => {
                debug!("variable/req/ok: {}", req);
                Ok(MmVariableRequest {
                    data,
                    function: header.function,
                    parsed: Some(req),
                })
            }
            Err(e) => {
                debug!("variable/req/err: {}", e);
                Ok(MmVariableRequest {
                    data,
                    function: header.function,
                    parsed: None,
                })
            }
        }
    }

    fn response(&self, status: Status, reply: Option<&[u8]>) -> Vec<u8> {
        let mut status = status;

        let limit = self.data.len() - core::mem::size_of::<MmVariableHeader>();
        if let Some(r) = reply {
            if limit < r.len() {
                status = Status::BUFFER_TOO_SMALL;
            }
        }

        debug!(
            "variable/rsp: {}, {} data byte(s) ({} available)",
            status,
            if let Some(r) = reply { r.len() } else { 0 },
            limit,
        );

        let hdr = MmVariableHeader {
            function: self.function,
            status: status.0 as u64,
        };
        let mut rsp = hdr.as_bytes().to_vec();
        if let Some(r) = reply {
            if limit < r.len() {
                rsp.extend_from_slice(&r[..limit]);
            } else {
                rsp.extend_from_slice(r);
            }
        }

        rsp
    }

    fn get_variable(&self, store: &EfiVarStore, guid: &Guid, name: &CStr16) -> Vec<u8> {
        let n = String::from(name);
        let res = store.get(&n, guid);
        if let Err(e) = res {
            debug!("variable/get/error: {e}");
            return self.response(e.status(), None);
        }

        let var = res.unwrap();
        let access = MmVariableAccess {
            guid: guid.to_bytes(),
            name_size: name.num_bytes() as u64,
            data_size: var.data.len() as u64,
            attributes: var.attr,
        };
        let mut reply = Vec::new();
        reply.extend_from_slice(access.as_bytes());
        reply.extend_from_slice(name.as_bytes());
        reply.extend_from_slice(&var.data);
        self.response(Status::SUCCESS, Some(&reply))
    }

    fn get_next_variable_name(&self, store: &EfiVarStore, guid: &Guid, name: &CStr16) -> Vec<u8> {
        let n = String::from(name);
        let res = store.get_next(&n, guid);
        if let Err(e) = res {
            return self.response(e.status(), None);
        }
        let (next_name, next_guid) = res.unwrap();
        debug!("variable/next: {} -> {}", name, next_name);

        let mut buffer: Vec<u16> = vec![0; next_name.len() + 1];
        let Ok(cstr16) = CStr16::from_str_with_buf(next_name, &mut buffer) else {
            // should not happen
            return self.response(Status::DEVICE_ERROR, None);
        };

        let next_var = MmNextVariable {
            guid: next_guid.to_bytes(),
            name_size: cstr16.num_bytes() as u64,
        };
        let mut reply = Vec::new();
        reply.extend_from_slice(next_var.as_bytes());
        reply.extend_from_slice(cstr16.as_bytes());
        self.response(Status::SUCCESS, Some(&reply))
    }

    fn set_variable(
        &self,
        store: &mut EfiVarStore,
        guid: &Guid,
        name: &CStr16,
        attr: &EfiVarAttr,
        data: &[u8],
    ) -> Vec<u8> {
        let var = EfiVar {
            guid: *guid,
            name: String::from(name),
            attr: *attr,
            data: data.to_vec(),
        };
        let res = store.set(var);
        if let Err(e) = res {
            debug!("variable/set/error: {e}");
            return self.response(e.status(), None);
        }
        self.response(Status::SUCCESS, None)
    }

    fn lock_variable(&self, _store: &EfiVarStore, _guid: &Guid, _name: &CStr16) -> Vec<u8> {
        debug!("TODO: implement lock-variable");
        self.response(Status::SUCCESS, None)
    }

    fn ready_to_boot(&self, store: &mut EfiVarStore) -> Vec<u8> {
        store.ready_to_boot();
        if false {
            self.response(Status::SUCCESS, None)
        } else {
            // qemu compatible
            Vec::new()
        }
    }

    fn exit_boot_service(&self, store: &mut EfiVarStore) -> Vec<u8> {
        store.exit_boot_service();
        if false {
            self.response(Status::SUCCESS, None)
        } else {
            // qemu compatible
            Vec::new()
        }
    }

    fn get_payload_size(&self) -> Vec<u8> {
        let reply = MmGetPayloadSize {
            payload_size: 64 * 1024,
        };
        self.response(Status::SUCCESS, Some(reply.as_bytes()))
    }

    fn process(&self, store: &mut EfiVarStore) -> Vec<u8> {
        if self.parsed.is_none() {
            error!("variable: req parse error");
            panic!();
        }
        let parsed = self.parsed.as_ref().unwrap();
        match parsed {
            MmVariableParsedRequest::GetVariable { guid, name } => {
                self.get_variable(store, guid, name)
            }
            MmVariableParsedRequest::GetNextVariableName { guid, name } => {
                self.get_next_variable_name(store, guid, name)
            }
            MmVariableParsedRequest::SetVariable {
                guid,
                name,
                attr,
                data,
            } => self.set_variable(store, guid, name, attr, data),
            MmVariableParsedRequest::ReadyToBoot => self.ready_to_boot(store),
            MmVariableParsedRequest::ExitBootService => self.exit_boot_service(store),
            MmVariableParsedRequest::LockVariable { guid, name } => {
                self.lock_variable(store, guid, name)
            }
            MmVariableParsedRequest::GetPayloadSize => self.get_payload_size(),
            _ => {
                error!("variable/unknown: {}", parsed);
                panic!();
            }
        }
    }
}

pub fn variable_request(store: &mut EfiVarStore, req: &[u8]) -> Vec<u8> {
    let res = MmVariableRequest::new(req);
    if let Err(e) = res.as_ref() {
        error!("{e}");
        return Vec::new();
    }

    let vreq = res.unwrap();
    vreq.process(store)
}
