//!
//! hob list types
//!
use core::ffi::c_void;

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
#[derive(Debug)]
pub struct HobHeader {
    pub hob_type: u16,
    pub hob_length: u16,
    _reserved: u32,
}

#[derive(Debug)]
pub enum Hob {
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
