use core::mem::size_of;
use log::debug;
use uguid::{guid, Guid};
use zerocopy::{Immutable, IntoBytes};

use igvm_defs::PAGE_SIZE_4K;

// edk2 = { 0x3dd177ff, 0xb632, 0x4e25, { 0xbe, 0xf3, 0x06, 0x50, 0x63, 0xd5, 0x5f, 0xc4 }}
pub const EFI_IGVM_DATA_HOB_GUID: Guid = guid!("3dd177ff-b632-4e25-bef3-065063d55fc4");

#[repr(u32)]
#[derive(Debug, IntoBytes, Immutable)]
pub enum EfiIgvmDataType {
    Pk = 0x100,
    Kek = 0x101,
    Db = 0x102,
    Dbx = 0x103,
    Shim = 0x200,
    Kernel = 0x201,
}

#[repr(C)]
#[derive(Debug, IntoBytes, Immutable)]
pub struct EfiIgvmDataHob {
    hob_type: u16,
    hob_length: u16,
    _reserved: u32,
    guid_ext: [u8; 16],
    address: u64,
    length: u64,
    data_type: EfiIgvmDataType,
    data_flags: u32,
}

impl EfiIgvmDataHob {
    pub fn new(address: usize, length: usize, data_type: EfiIgvmDataType) -> Self {
        Self {
            hob_type: 0x0004, // EFI_HOB_TYPE_GUID_EXTENSION
            hob_length: size_of::<EfiIgvmDataHob>() as u16,
            _reserved: 0,
            guid_ext: EFI_IGVM_DATA_HOB_GUID.to_bytes(),
            address: address as u64,
            length: length as u64,
            data_type,
            data_flags: 0,
        }
    }
}

pub struct EfiIgvmData<'b> {
    hob: EfiIgvmDataHob,
    addr: usize,
    blob: &'b [u8],
    measured: bool,
}

pub struct EfiIgvmDataList<'b> {
    base: usize,
    data: Vec<EfiIgvmData<'b>>,
}

impl<'b> EfiIgvmDataList<'b> {
    pub fn new(base: usize) -> Self {
        Self {
            base,
            data: Vec::new(),
        }
    }

    pub fn add(&mut self, blob: &'b [u8], data_type: EfiIgvmDataType, measured: bool) {
        debug!(
            "add {data_type:?}, 0x{:x} bytes, at 0x{:x}{}",
            blob.len(),
            self.base,
            if measured { "" } else { ", unmeasured" },
        );
        let hob = EfiIgvmDataHob::new(self.base, blob.len(), data_type);
        let addr = self.base;
        let data = EfiIgvmData {
            hob,
            addr,
            blob,
            measured,
        };
        self.data.push(data);
        self.base += blob.len().next_multiple_of(PAGE_SIZE_4K as usize);
    }

    pub fn hobs(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        for d in &self.data {
            blob.extend_from_slice(d.hob.as_bytes());
        }

        // EFI_HOB_TYPE_END_OF_HOB_LIST
        blob.extend_from_slice(&[0xff, 0xff]);

        blob
    }

    pub fn blobs(&self, measured: bool) -> Vec<(usize, &'b [u8])> {
        self.data
            .iter()
            .filter(|d| d.measured == measured)
            .map(|d| (d.addr, d.blob))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
