//!
//! EfiSmmVariableProtocol reimplementation
//!
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Display;
use log::{debug, error};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use uefi::{CString16, Error, Status};
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr, EfiVarId};
use virtfw_libefi::guids::RawGuid;

use crate::mm::util;
use crate::store::EfiVarStore;

const GET_VARIABLE: u64 = 1;
const GET_NEXT_VARIABLE_NAME: u64 = 2;
const SET_VARIABLE: u64 = 3;
const QUERY_VARIABLE_INFO: u64 = 4;
const READY_TO_BOOT: u64 = 5;
const EXIT_BOOT_SERVICE: u64 = 6;
const LOCK_VARIABLE: u64 = 8;
const GET_PAYLOAD_SIZE: u64 = 11;

// SMM_VARIABLE_COMMUNICATE_HEADER
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
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
    pub fn new_from_nr(nr: u64) -> Self {
        Self {
            function: nr,
            status: 0,
        }
    }
}

// SMM_VARIABLE_COMMUNICATE_ACCESS_VARIABLE
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
struct MmVariableAccess {
    guid: RawGuid,
    data_size: u64,
    name_size: u64,
    attributes: EfiVarAttr,
    // Name
    // Data
}

impl MmVariableAccess {
    fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read MmVariableAccess",
        )))
    }
}

// SMM_VARIABLE_COMMUNICATE_GET_NEXT_VARIABLE_NAME
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
struct MmNextVariable {
    guid: RawGuid,
    name_size: u64,
    // Name
}

impl MmNextVariable {
    fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read MmNextVariable",
        )))
    }
}

// SMM_VARIABLE_COMMUNICATE_QUERY_VARIABLE_INFO
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
struct MmVariableInfo {
    max_storage_size: u64,
    free_storage_size: u64,
    max_variable_size: u64,
    attributes: u32,
}

// SMM_VARIABLE_COMMUNICATE_GET_PAYLOAD_SIZE
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
struct MmGetPayloadSize {
    payload_size: u64,
}

// parsed request
#[derive(Debug, Eq, PartialEq)]
pub enum MmVariableParsedRequest {
    GetVariable(EfiVarId),
    GetNextVariableName(EfiVarId),
    SetVariable(EfiVar),
    QueryVariableInfo,
    ReadyToBoot,
    ExitBootService,
    LockVariable(EfiVarId),
    GetPayloadSize,
    Unknown { function: u64 },
}

impl MmVariableParsedRequest {
    pub fn new(function: u64, body: &[u8]) -> Result<Self, Error<&'static str>> {
        let req = match function {
            GET_VARIABLE => {
                let (access, namedata) = MmVariableAccess::new(body)?;
                let ns = access.name_size as usize;
                let Some(slice) = &namedata.get(0..ns) else {
                    return Err(Error::new(Status::BAD_BUFFER_SIZE, "invalid name size"));
                };
                let name = util::cstr16_from_bytes_with_nul(slice)?;
                let guid = access.guid.into();
                let id = EfiVarId::new(String::from(name), guid);
                MmVariableParsedRequest::GetVariable(id)
            }
            GET_NEXT_VARIABLE_NAME | LOCK_VARIABLE => {
                let (next, namebytes) = MmNextVariable::new(body)?;
                let ns = next.name_size as usize;
                let Some(slice) = &namebytes.get(0..ns) else {
                    return Err(Error::new(Status::BAD_BUFFER_SIZE, "invalid name size"));
                };
                let name = util::cstr16_from_bytes_until_nul(slice)?;
                let guid = next.guid.into();
                let id = EfiVarId::new(String::from(name), guid);
                if function == GET_NEXT_VARIABLE_NAME {
                    MmVariableParsedRequest::GetNextVariableName(id)
                } else {
                    MmVariableParsedRequest::LockVariable(id)
                }
            }
            SET_VARIABLE => {
                let (access, namedata) = MmVariableAccess::new(body)?;
                let ns = access.name_size as usize;
                let ds = access.data_size as usize;
                let Some(slice) = &namedata.get(0..ns) else {
                    return Err(Error::new(Status::BAD_BUFFER_SIZE, "invalid name size"));
                };
                let name = util::cstr16_from_bytes_with_nul(slice)?;
                let Some(data) = &namedata.get(ns..ns + ds) else {
                    return Err(Error::new(Status::BAD_BUFFER_SIZE, "invalid data size"));
                };
                let guid = access.guid.into();
                let var =
                    EfiVar::new_from_slice(&guid, &String::from(name), access.attributes, data);
                MmVariableParsedRequest::SetVariable(var)
            }
            QUERY_VARIABLE_INFO => MmVariableParsedRequest::QueryVariableInfo,
            READY_TO_BOOT => MmVariableParsedRequest::ReadyToBoot,
            EXIT_BOOT_SERVICE => MmVariableParsedRequest::ExitBootService,
            GET_PAYLOAD_SIZE => MmVariableParsedRequest::GetPayloadSize,
            _ => MmVariableParsedRequest::Unknown { function },
        };
        Ok(req)
    }

    fn serialize_simple(nr: u64) -> Vec<u8> {
        MmVariableHeader::new_from_nr(nr).as_bytes().to_vec()
    }

    fn serialize_next(id: &EfiVarId, nr: u64) -> Vec<u8> {
        let hdr = MmVariableHeader::new_from_nr(nr);
        let name = CString16::try_from(id.name.as_str()).unwrap_or(CString16::new());
        let next = MmNextVariable {
            guid: id.guid.into(),
            name_size: name.num_bytes() as u64,
        };
        let mut blob = Vec::new();
        blob.extend_from_slice(hdr.as_bytes());
        blob.extend_from_slice(next.as_bytes());
        blob.extend_from_slice(name.as_bytes());
        blob
    }
}

impl From<&MmVariableParsedRequest> for Vec<u8> {
    fn from(value: &MmVariableParsedRequest) -> Vec<u8> {
        match value {
            MmVariableParsedRequest::GetVariable(id) => {
                let hdr = MmVariableHeader::new_from_nr(GET_VARIABLE);
                let name = CString16::try_from(id.name.as_str()).unwrap_or(CString16::new());
                let access = MmVariableAccess {
                    guid: id.guid.into(),
                    attributes: EfiVarAttr::new(),
                    name_size: name.num_bytes() as u64,
                    data_size: 0,
                };
                let mut blob = Vec::new();
                blob.extend_from_slice(hdr.as_bytes());
                blob.extend_from_slice(access.as_bytes());
                blob.extend_from_slice(name.as_bytes());
                blob
            }
            MmVariableParsedRequest::SetVariable(var) => {
                let hdr = MmVariableHeader::new_from_nr(SET_VARIABLE);
                let name = CString16::try_from(var.name.as_str()).unwrap_or(CString16::new());
                let access = MmVariableAccess {
                    guid: var.guid.into(),
                    attributes: var.attr,
                    name_size: name.num_bytes() as u64,
                    data_size: var.data.len() as u64,
                };
                let mut blob = Vec::new();
                blob.extend_from_slice(hdr.as_bytes());
                blob.extend_from_slice(access.as_bytes());
                blob.extend_from_slice(name.as_bytes());
                blob.extend_from_slice(var.data.as_bytes());
                blob
            }
            MmVariableParsedRequest::GetNextVariableName(id) => {
                MmVariableParsedRequest::serialize_next(id, GET_NEXT_VARIABLE_NAME)
            }
            MmVariableParsedRequest::LockVariable(id) => {
                MmVariableParsedRequest::serialize_next(id, LOCK_VARIABLE)
            }
            MmVariableParsedRequest::QueryVariableInfo => {
                MmVariableParsedRequest::serialize_simple(QUERY_VARIABLE_INFO)
            }
            MmVariableParsedRequest::ReadyToBoot => {
                MmVariableParsedRequest::serialize_simple(READY_TO_BOOT)
            }
            MmVariableParsedRequest::ExitBootService => {
                MmVariableParsedRequest::serialize_simple(EXIT_BOOT_SERVICE)
            }
            MmVariableParsedRequest::GetPayloadSize => {
                MmVariableParsedRequest::serialize_simple(GET_PAYLOAD_SIZE)
            }
            MmVariableParsedRequest::Unknown { function } => {
                MmVariableParsedRequest::serialize_simple(*function)
            }
        }
    }
}

impl Display for MmVariableParsedRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            MmVariableParsedRequest::GetVariable(id) => {
                write!(f, "request/var: get-variable {}", id.name)
            }
            MmVariableParsedRequest::GetNextVariableName(id) => {
                write!(f, "request/var: get-next-variable-name {}", id.name)
            }
            MmVariableParsedRequest::SetVariable(var) => {
                write!(
                    f,
                    "request/var: set-variable {}  {}  {} data bytes",
                    var.name,
                    var.attr,
                    var.data.len()
                )
            }
            MmVariableParsedRequest::QueryVariableInfo => {
                write!(f, "request/var: query-variable-info")
            }
            MmVariableParsedRequest::ReadyToBoot => {
                write!(f, "request/var: ready-to-boot")
            }
            MmVariableParsedRequest::ExitBootService => {
                write!(f, "request/var: exit-boot-service")
            }
            MmVariableParsedRequest::LockVariable(id) => {
                write!(f, "request/var: lock-variable {}", id.name)
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
    pub parsed: Option<MmVariableParsedRequest>,
}

impl<'a> MmVariableRequest<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error<&'static str>> {
        let (header, body) = MmVariableHeader::new(data)?;
        let req = match MmVariableParsedRequest::new(header.function, body) {
            Ok(req) => {
                debug!("variable/req/ok: {req}");
                Some(req)
            }
            Err(e) => {
                debug!("variable/req/err: {e}");
                None
            }
        };
        Ok(MmVariableRequest {
            data,
            function: header.function,
            parsed: req,
        })
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

    fn get_variable(&self, store: &EfiVarStore, id: &EfiVarId) -> Vec<u8> {
        let res = store.get(&id.name, &id.guid);
        if let Err(e) = res {
            debug!("variable/get/error: {e}");
            return self.response(e.status(), None);
        }

        let Ok(cstr16) = CString16::try_from(id.name.as_str()) else {
            return self.response(Status::DEVICE_ERROR, None);
        };

        let var = res.unwrap();
        let access = MmVariableAccess {
            guid: id.guid.into(),
            name_size: cstr16.num_bytes() as u64,
            data_size: var.data.len() as u64,
            attributes: var.attr,
        };
        let mut reply = Vec::new();
        reply.extend_from_slice(access.as_bytes());
        reply.extend_from_slice(cstr16.as_bytes());
        reply.extend_from_slice(&var.data);
        self.response(Status::SUCCESS, Some(&reply))
    }

    fn get_next_variable_name(&self, store: &EfiVarStore, id: &EfiVarId) -> Vec<u8> {
        let res = store.get_next(&id.name, &id.guid);
        if let Err(e) = res {
            return self.response(e.status(), None);
        }
        let (next_name, next_guid) = res.unwrap();
        debug!("variable/next: {} -> {}", id.name, next_name);

        let Ok(cstr16) = CString16::try_from(next_name) else {
            return self.response(Status::DEVICE_ERROR, None);
        };

        let next_var = MmNextVariable {
            guid: next_guid.into(),
            name_size: cstr16.num_bytes() as u64,
        };
        let mut reply = Vec::new();
        reply.extend_from_slice(next_var.as_bytes());
        reply.extend_from_slice(cstr16.as_bytes());
        self.response(Status::SUCCESS, Some(&reply))
    }

    fn set_variable(&self, store: &mut EfiVarStore, var: &EfiVar) -> Vec<u8> {
        let res = store.set(var.clone());
        if let Err(e) = res {
            debug!("variable/set/error: {e}");
            return self.response(e.status(), None);
        }
        self.response(Status::SUCCESS, None)
    }

    fn query_variable_info(&self, _store: &EfiVarStore) -> Vec<u8> {
        let reply = MmVariableInfo {
            max_storage_size: 256 * 1024, // qemu default
            free_storage_size: 0,         // FIXME
            max_variable_size: 64 * 1024, // qemu default
            attributes: 0,
        };
        self.response(Status::SUCCESS, Some(reply.as_bytes()))
    }

    fn lock_variable(&self, store: &mut EfiVarStore, id: &EfiVarId) -> Vec<u8> {
        let res = store.lock(&id.name, &id.guid);
        if let Err(e) = res {
            debug!("variable/lock/error: {e}");
            return self.response(e.status(), None);
        }
        self.response(Status::SUCCESS, None)
    }

    fn ready_to_boot(&self, store: &mut EfiVarStore) -> Vec<u8> {
        store.ready_to_boot();
        self.response(Status::SUCCESS, None)
    }

    fn exit_boot_service(&self, store: &mut EfiVarStore) -> Vec<u8> {
        store.exit_boot_service();
        self.response(Status::SUCCESS, None)
    }

    fn get_payload_size(&self) -> Vec<u8> {
        let reply = MmGetPayloadSize {
            payload_size: 64 * 1024, // qemu max
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
            MmVariableParsedRequest::GetVariable(id) => self.get_variable(store, id),
            MmVariableParsedRequest::GetNextVariableName(id) => {
                self.get_next_variable_name(store, id)
            }
            MmVariableParsedRequest::SetVariable(var) => self.set_variable(store, var),
            MmVariableParsedRequest::QueryVariableInfo => self.query_variable_info(store),
            MmVariableParsedRequest::ReadyToBoot => self.ready_to_boot(store),
            MmVariableParsedRequest::ExitBootService => self.exit_boot_service(store),
            MmVariableParsedRequest::LockVariable(id) => self.lock_variable(store, id),
            MmVariableParsedRequest::GetPayloadSize => self.get_payload_size(),
            _ => {
                error!("variable/unknown: {parsed}");
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
