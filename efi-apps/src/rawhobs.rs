//!
//! Raw C/EFI hob list types
//!
use zerocopy::{FromBytes, IntoBytes};

pub const EFI_RESOURCE_SYSTEM_MEMORY: u32 = 0x00000000;
pub const EFI_RESOURCE_MEMORY_MAPPED_IO: u32 = 0x00000001;
pub const EFI_RESOURCE_IO: u32 = 0x00000002;
pub const EFI_RESOURCE_FIRMWARE_DEVICE: u32 = 0x00000003;
pub const EFI_RESOURCE_MEMORY_MAPPED_IO_PORT: u32 = 0x00000004;
pub const EFI_RESOURCE_MEMORY_RESERVED: u32 = 0x00000005;
pub const EFI_RESOURCE_IO_RESERVED: u32 = 0x00000006;
pub const EFI_RESOURCE_MEMORY_UNACCEPTED: u32 = 0x00000007;
pub const EFI_RESOURCE_MAX_MEMORY_TYPE: u32 = 0x00000008;

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes)]
pub struct HobResourceDescriptor {
    pub owner: [u8; 16], // guid
    pub res_type: u32,
    pub res_attr: u32,
    pub address: u64,
    pub length: u64,
}

impl HobResourceDescriptor {
    pub fn typename(&self) -> &'static str {
        match self.res_type {
            EFI_RESOURCE_SYSTEM_MEMORY => "system-memory",
            EFI_RESOURCE_MEMORY_MAPPED_IO => "mmio",
            EFI_RESOURCE_IO => "io",
            EFI_RESOURCE_FIRMWARE_DEVICE => "fw-device",
            EFI_RESOURCE_MEMORY_MAPPED_IO_PORT => "mmio-port",
            EFI_RESOURCE_MEMORY_RESERVED => "memory-reserved",
            EFI_RESOURCE_IO_RESERVED => "io-reserved",
            EFI_RESOURCE_MEMORY_UNACCEPTED => "unaccepted",
            _ => "unknown",
        }
    }
}
