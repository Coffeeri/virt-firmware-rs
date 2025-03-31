//! misc ef data types

use bitfield_struct::bitfield;
use uuid::Uuid;

#[bitfield(u32)]
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
    pub guid: Uuid,
}

#[derive(Debug)]
pub struct EfiVar {
    pub name: String,
    pub guid: Uuid,
    pub attr: EfiVarAttr,
    pub data: Vec<u8>,
}
