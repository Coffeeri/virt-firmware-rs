//!
//! hob list types
//!
use core::ffi::c_void;
use zerocopy::FromBytes;

use uefi::system::with_config_table;
use uefi::table::cfg::ConfigTableEntry;
use uefi::Guid;

use virtfw_libefi::guids::{pretty_name, HandOffBlockListCfg};

use crate::rawhobs::{
    HobHeader, HobMemoryAllocation, HobResourceDescriptor, EFI_HOB_TYPE_END_OF_HOB_LIST,
    EFI_HOB_TYPE_GUID_EXTENSION, EFI_HOB_TYPE_MEMORY_ALLOCATION, EFI_HOB_TYPE_RESOURCE_DESCRIPTOR,
};

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
