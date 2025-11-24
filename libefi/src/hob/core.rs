use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable)]
pub struct EfiHobHeader {
    pub hob_type: u16,
    pub hob_length: u16,
    pub _reserved: u32,
}
