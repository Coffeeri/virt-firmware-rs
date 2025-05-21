//!
//! hob list types
//!
use core::ffi::c_void;
use zerocopy::{FromBytes, IntoBytes};

use uefi::system::with_config_table;
use uefi::table::cfg::ConfigTableEntry;
use uefi::Guid;

use virtfw_libefi::guids::{pretty_name, HandOffBlockListCfg};

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
#[derive(Debug, FromBytes, IntoBytes)]
pub struct HobHeader {
    pub hob_type: u16,
    pub hob_length: u16,
    _reserved: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes)]
pub struct HobMemoryAllocation {
    pub name: [u8; 16], // guid
    pub address: u64,
    pub length: u64,
    pub mem_type: u32,
    _reserved: [u8; 4],
}

impl HobMemoryAllocation {
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

#[derive(Debug)]
pub enum Hob {
    MemoryAllocation(HobMemoryAllocation),
    ResourceDescriptor(HobResourceDescriptor),
    OtherType { hob_type: u16 },
    OtherGuid { hob_guid: Guid },
}

impl Hob {
    fn new_guid(b: &[u8]) -> Hob {
        let bytes: [u8; 16] = b[0..16].try_into().unwrap();
        let guid = Guid::from_bytes(bytes);
        Hob::OtherGuid { hob_guid: guid }
    }

    pub fn new(h: &HobHeader, b: &[u8]) -> Hob {
        match h.hob_type {
            EFI_HOB_TYPE_MEMORY_ALLOCATION => {
                let m = HobMemoryAllocation::read_from_prefix(b);
                Hob::MemoryAllocation(m.unwrap().0)
            }
            EFI_HOB_TYPE_RESOURCE_DESCRIPTOR => {
                let r = HobResourceDescriptor::read_from_prefix(b);
                Hob::ResourceDescriptor(r.unwrap().0)
            }
            EFI_HOB_TYPE_GUID_EXTENSION => Self::new_guid(b),
            _ => Hob::OtherType {
                hob_type: h.hob_type,
            },
        }
    }
}

impl core::fmt::Display for Hob {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Hob::MemoryAllocation(m) => {
                write!(
                    f,
                    "memory allocation: 0x{:08x} +0x{:08x} type {}",
                    m.address,
                    m.length,
                    m.typename()
                )
            }
            Hob::ResourceDescriptor(r) => {
                write!(
                    f,
                    "resource descriptor: 0x{:08x} +0x{:08x} attr 0x{:04x} type {}",
                    r.address,
                    r.length,
                    r.res_attr,
                    r.typename(),
                )
            }
            Hob::OtherType { hob_type } => write!(f, "other type: {}", hob_type),
            Hob::OtherGuid { hob_guid } => write!(f, "other guid: {}", pretty_name(hob_guid)),
        }
    }
}

pub struct HobList {
    address: *const c_void,
    offset: usize,
}

impl HobList {
    fn findentry(entries: &[ConfigTableEntry]) -> Option<ConfigTableEntry> {
        let e = entries.iter().find(|e| e.guid == HandOffBlockListCfg)?;
        Some(e.clone())
    }

    pub fn new() -> Option<Self> {
        let entry = with_config_table(Self::findentry)?;
        Some(HobList {
            address: entry.address,
            offset: 0,
        })
    }
}

impl Iterator for HobList {
    type Item = Hob;

    fn next(&mut self) -> Option<Hob> {
        let (h, b) = unsafe {
            let addr = self.address.add(self.offset);
            let h: &HobHeader = &*(addr as *const HobHeader);
            let b: &[u8] =
                core::slice::from_raw_parts(addr.add(8) as *const u8, h.hob_length as usize - 8);
            (h, b)
        };
        if h.hob_type == EFI_HOB_TYPE_END_OF_HOB_LIST {
            return None;
        }
        self.offset += h.hob_length as usize;
        Some(Hob::new(h, b))
    }
}
