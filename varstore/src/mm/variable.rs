//!
//! [WIP] EfiSmmVariableProtocol reimplementation
//!
use core::fmt::Display;
use uguid::Guid;
use zerocopy::FromBytes;
use zerocopy_derive::FromBytes;

use uefi::{CStr16, Error, Status};

use virtfw_libefi::efivar::types::EfiVarAttr;

use crate::mm::util;

pub const GET_VARIABLE: u64 = 1;
pub const GET_NEXT_VARIABLE_NAME: u64 = 2;
pub const SET_VARIABLE: u64 = 3;
pub const QUERY_VARIABLE_INFO: u64 = 4;
pub const READY_TO_BOOT: u64 = 5;
pub const EXIT_BOOT_SERVICE: u64 = 6;
pub const LOCK_VARIABLE: u64 = 8;
pub const GET_PAYLOAD_SIZE: u64 = 11;

// SMM_VARIABLE_COMMUNICATE_HEADER
#[repr(C, packed)]
#[derive(Debug, FromBytes)]
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
#[derive(Debug, FromBytes)]
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
#[derive(Debug, FromBytes)]
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
#[derive(Debug, FromBytes)]
pub struct MmGetPayloadSize {
    pub payload_size: u64,
}

// parsed request
#[derive(Debug)]
pub enum MmVariableRequest<'a> {
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
    // LockVariable = 8,
    // GetPayloadSize = 11,
    Unknown {
        function: u64,
    },
}

impl<'a> MmVariableRequest<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error<&'static str>> {
        let (header, body) = MmVariableHeader::new(data)?;
        let req = match header.function {
            GET_VARIABLE => {
                let (access, namedata) = MmVariableAccess::new(body)?;
                let ns = access.name_size as usize;
                let name = util::cstr16_from_bytes_with_nul(&namedata[0..ns])?;
                let guid = Guid::from_bytes(access.guid);
                MmVariableRequest::GetVariable { guid, name }
            }
            GET_NEXT_VARIABLE_NAME => {
                let (next, namebytes) = MmNextVariable::new(body)?;
                let ns = next.name_size as usize;
                let name = util::cstr16_from_bytes_until_nul(&namebytes[0..ns])?;
                let guid = Guid::from_bytes(next.guid);
                MmVariableRequest::GetNextVariableName { guid, name }
            }
            SET_VARIABLE => {
                let (access, namedata) = MmVariableAccess::new(body)?;
                let ns = access.name_size as usize;
                let ds = access.data_size as usize;
                let name = util::cstr16_from_bytes_with_nul(&namedata[0..ns])?;
                let data = &namedata[ns..ns + ds];
                let guid = Guid::from_bytes(access.guid);
                MmVariableRequest::SetVariable {
                    guid,
                    name,
                    attr: access.attributes,
                    data,
                }
            }
            READY_TO_BOOT => MmVariableRequest::ReadyToBoot,
            EXIT_BOOT_SERVICE => MmVariableRequest::ExitBootService,
            _ => MmVariableRequest::Unknown {
                function: header.function,
            },
        };
        Ok(req)
    }
}

impl Display for MmVariableRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            MmVariableRequest::GetVariable { guid: _, name } => {
                write!(f, "request/var: get-variable {name}",)
            }
            MmVariableRequest::GetNextVariableName { guid: _, name } => {
                write!(f, "request/var: get-next-variable-name {name}",)
            }
            MmVariableRequest::SetVariable {
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
            MmVariableRequest::ReadyToBoot => {
                write!(f, "request/var: ready-to-boot")
            }
            MmVariableRequest::ExitBootService => {
                write!(f, "request/var: exit-boot-service")
            }
            MmVariableRequest::Unknown { function } => {
                write!(f, "request/var: unknown function #{function}")
            }
        }
    }
}
