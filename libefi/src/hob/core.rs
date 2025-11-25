use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::guids::RawGuid;

pub const EFI_HOB_TYPE_HANDOFF: u16 = 0x0001;
pub const EFI_HOB_TYPE_MEMORY_ALLOCATION: u16 = 0x0002;
pub const EFI_HOB_TYPE_RESOURCE_DESCRIPTOR: u16 = 0x0003;
pub const EFI_HOB_TYPE_GUID_EXTENSION: u16 = 0x0004;
pub const EFI_HOB_TYPE_FV: u16 = 0x0005;
pub const EFI_HOB_TYPE_CPU: u16 = 0x0006;
pub const EFI_HOB_TYPE_MEMORY_POOL: u16 = 0x0007;
pub const EFI_HOB_TYPE_FV2: u16 = 0x0009;
pub const EFI_HOB_TYPE_LOAD_PEIM_UNUSED: u16 = 0x000A;
pub const EFI_HOB_TYPE_UEFI_CAPSULE: u16 = 0x000B;
pub const EFI_HOB_TYPE_FV3: u16 = 0x000C;
pub const EFI_HOB_TYPE_UNUSED: u16 = 0xFFFE;
pub const EFI_HOB_TYPE_END_OF_HOB_LIST: u16 = 0xFFFF;

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable)]
pub struct EfiHobHeader {
    pub hob_type: u16,
    pub hob_length: u16,
    pub _reserved: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable)]
pub struct EfiHobMemoryAllocation {
    pub name: RawGuid,
    pub address: u64,
    pub length: u64,
    pub mem_type: u32,
    _reserved: [u8; 4],
}

impl EfiHobMemoryAllocation {
    pub fn typename(&self) -> &'static str {
        match self.mem_type {
            0 => "reserved",
            1 => "loader-code",
            2 => "loader-data",
            3 => "bs-code",
            4 => "bs-data",
            5 => "rt-code",
            6 => "rt-data",
            7 => "memory",
            8 => "unusable",
            9 => "acpi-reclaim",
            10 => "acpi-nvs",
            11 => "mmio",
            12 => "mmio-port",
            13 => "pal-code",
            14 => "persistent",
            15 => "unaccepted",
            _ => "unknown",
        }
    }
}
