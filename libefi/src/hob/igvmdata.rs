extern crate alloc;
use alloc::vec::Vec;

use core::mem::size_of;
use log::debug;
use uguid::{guid, Guid};
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::guids::RawGuid;
use crate::hob::core::{EfiHobHeader, EFI_HOB_TYPE_GUID_EXTENSION};

const PAGE_SIZE_4K: usize = 4096;

// edk2 = { 0x3dd177ff, 0xb632, 0x4e25, { 0xbe, 0xf3, 0x06, 0x50, 0x63, 0xd5, 0x5f, 0xc4 }}
pub const EFI_IGVM_DATA_HOB_GUID: Guid = guid!("3dd177ff-b632-4e25-bef3-065063d55fc4");

#[repr(u32)]
#[derive(Debug, IntoBytes, Immutable, Clone, Copy)]
pub enum EfiIgvmDataType {
    Pk = 0x100,
    Kek = 0x101,
    Db = 0x102,
    Dbx = 0x103,
    Shim = 0x200,
    Kernel = 0x201,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, Clone)]
pub struct EfiIgvmDataHob {
    pub hdr: EfiHobHeader,
    pub guid_ext: RawGuid,
    pub address: u64,
    pub length: u64,
    pub data_type: u32,
    pub data_flags: u32,
}

impl EfiIgvmDataHob {
    pub fn new(address: usize, length: usize, data_type: EfiIgvmDataType) -> Self {
        let hdr = EfiHobHeader::new(
            EFI_HOB_TYPE_GUID_EXTENSION,
            size_of::<EfiIgvmDataHob>() as u16,
        );
        Self {
            hdr,
            guid_ext: EFI_IGVM_DATA_HOB_GUID.into(),
            address: address as u64,
            length: length as u64,
            data_type: data_type as u32,
            data_flags: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.hdr.hob_type != EFI_HOB_TYPE_GUID_EXTENSION {
            return false;
        }
        if self.hdr.hob_length != size_of::<EfiIgvmDataHob>() as u16 {
            return false;
        }
        if self.guid_ext != EFI_IGVM_DATA_HOB_GUID.into() {
            return false;
        }
        true
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
        self.base += blob.len().next_multiple_of(PAGE_SIZE_4K);
    }

    pub fn hobs(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        for d in &self.data {
            blob.extend_from_slice(d.hob.as_bytes());
        }

        let eol = EfiHobHeader::end_of_list();
        blob.extend_from_slice(eol.as_bytes());

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
