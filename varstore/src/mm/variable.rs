use core::fmt::Display;
use zerocopy::error::SizeError;
use zerocopy::FromBytes;
use zerocopy_derive::FromBytes;

use virtfw_libefi::efivar::types::EfiVarAttr;

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

// SMM_VARIABLE_COMMUNICATE_GET_NEXT_VARIABLE_NAME
#[repr(C, packed)]
#[derive(Debug, FromBytes)]
pub struct MmNextVariable {
    pub guid: [u8; 16],
    pub name_size: u64,
    // Name
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

#[derive(Debug)]
pub enum MmVariableRequest {
    // GetVariable { access: MmVariableAccess },
    // GetNextVariableName = 2,
    // SetVariable = 3,
    // QueryVariableInfo = 4,
    ReadyToBoot,
    ExitBootService,
    // LockVariable = 8,
    // GetPayloadSize = 11,
    Unknown { function: u64 },
}

impl MmVariableRequest {
    pub fn new(data: &[u8]) -> Result<Self, SizeError<&[u8], MmVariableHeader>> {
        let (header, _body) = MmVariableHeader::read_from_prefix(data)?;
        let req = match header.function {
            // GET_VARIABLE => {
            // let (access, namedata) = MmVariableAccess::read_from_prefix(body)?;
            // MmVariableRequest::GetVariable { access }
            // }
            READY_TO_BOOT => MmVariableRequest::ReadyToBoot,
            EXIT_BOOT_SERVICE => MmVariableRequest::ExitBootService,
            _ => MmVariableRequest::Unknown {
                function: header.function,
            },
        };
        Ok(req)
    }
}

impl Display for MmVariableRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            // MmVariableRequest::GetVariable { access } => {
            // write!(f, "request/var: get-variable {:?}", access)
            // }
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
