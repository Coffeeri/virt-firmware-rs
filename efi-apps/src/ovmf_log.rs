use zerocopy::{FromBytes, IntoBytes};

pub const MAGIC1: u64 = 0x3167646d666d766f; // "ovmfmdg1"
pub const MAGIC2: u64 = 0x3267646d666d766f; // "ovmfmdg2"

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Clone)]
pub struct MemDebugLogHeader {
    pub magic1: u64,
    pub magic2: u64,
    pub header_size: u64,
    pub log_size: u64,
    _lock: u64,
    pub head_offset: u64,
    pub tail_offset: u64,
    pub truncated: u64,
    pub fw_version: [u8; 128],
}

pub struct MemDebugLog<'l> {
    pub hdr: MemDebugLogHeader,
    pub log: &'l [u8],
}

impl<'l> MemDebugLog<'l> {
    pub fn new(addr: u64) -> Option<Self> {
        let hdr: &MemDebugLogHeader = unsafe { &*(addr as *const MemDebugLogHeader) };

        if hdr.magic1 != MAGIC1 || hdr.magic2 != MAGIC2 {
            return None;
        }

        let log: &[u8] = unsafe {
            core::slice::from_raw_parts(
                (addr + hdr.header_size) as *const u8,
                hdr.log_size as usize,
            )
        };

        Some(Self {
            hdr: hdr.clone(),
            log,
        })
    }
}
