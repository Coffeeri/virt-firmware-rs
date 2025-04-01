//! misc ef data types

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use zerocopy_derive::FromBytes;

use bitfield_struct::bitfield;
use uguid::Guid;

#[bitfield(u32)]
#[derive(FromBytes)]
pub struct EfiVarAttr {
    pub non_volatile: bool,
    pub bootservice_access: bool,
    pub runtime_access: bool,
    pub hardware_error: bool,

    pub auth_wr_access: bool, // deprecated
    pub time_auth_wr_access: bool,
    pub append_write: bool,

    #[bits(25)]
    pub reserved: u32,
}

impl EfiVarAttr {
    pub fn new_nv_bs_rt() -> EfiVarAttr {
        EfiVarAttr::new()
            .with_non_volatile(true)
            .with_bootservice_access(true)
            .with_runtime_access(true)
    }
}

#[derive(Debug)]
pub struct EfiVarId {
    pub name: String,
    pub guid: Guid,
}

#[derive(Debug)]
pub struct EfiVar {
    pub name: String,
    pub guid: Guid,
    pub attr: EfiVarAttr,
    pub data: Vec<u8>,
}
