//! qemu fw_cfg access
pub mod mmio;
pub mod x86;

#[cfg(feature = "alloc")]
pub mod acpi;
#[cfg(feature = "alloc")]
pub mod smbios;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use log::debug;

use core::ptr;

use crate::size::*;

pub const ITEM_SIGNATURE: u16 = 0x00;
pub const ITEM_VERSION: u16 = 0x01;
pub const ITEM_INITRD_SIZE: u16 = 0xb;
pub const ITEM_INITRD_DATA: u16 = 0x12;
pub const ITEM_CMDLINE_SIZE: u16 = 0x14;
pub const ITEM_CMDLINE_DATA: u16 = 0x15;
pub const ITEM_FILEDIR: u16 = 0x19;

pub const FILENAME_LEN: usize = 56;

//const DMA_CTL_ERROR: u32 = 0x01;
const DMA_CTL_READ: u32 = 0x02;
const DMA_CTL_SKIP: u32 = 0x04;
const DMA_CTL_SELECT: u32 = 0x08;
const DMA_CTL_WRITE: u32 = 0x10;

#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct FwCfgFile {
    pub size: u32,
    pub item: u16,
    pub name: String,
}

#[repr(C)]
struct FwCfgDma {
    control: u32,
    length: u32,
    address: u64,
}

pub trait FwCfg {
    fn select(&self, item: u16);
    fn read8(&self) -> u8;
    fn trigger_dma(&self, addr: usize);

    #[cfg(feature = "alloc")]
    fn read_bytes(&self, count: usize) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        for _ in 0..count {
            data.push(self.read8());
        }
        data
    }

    fn read_bytes_slice(&self, dest: &mut [u8]) {
        for byte in dest {
            *byte = self.read8();
        }
    }

    fn read16_le(&self) -> u16 {
        let b1 = self.read8() as u16;
        let b2 = self.read8() as u16;
        b1 | (b2 << 8)
    }

    fn read32_le(&self) -> u32 {
        let w1 = self.read16_le() as u32;
        let w2 = self.read16_le() as u32;
        w1 | (w2 << 16)
    }

    fn read64_le(&self) -> u64 {
        let l1 = self.read32_le() as u64;
        let l2 = self.read32_le() as u64;
        l1 | (l2 << 32)
    }

    fn read16_be(&self) -> u16 {
        let b1 = self.read8() as u16;
        let b2 = self.read8() as u16;
        b2 | (b1 << 8)
    }

    fn read32_be(&self) -> u32 {
        let w1 = self.read16_be() as u32;
        let w2 = self.read16_be() as u32;
        w2 | (w1 << 16)
    }

    fn read64_be(&self) -> u64 {
        let l1 = self.read32_be() as u64;
        let l2 = self.read32_be() as u64;
        l2 | (l1 << 32)
    }

    fn detect(&self) -> bool {
        self.select(ITEM_SIGNATURE);
        let sig = self.read32_le();
        if sig != 0x554d4551 {
            return false;
        }

        #[cfg(feature = "alloc")]
        {
            self.select(ITEM_VERSION);
            let ver = self.read32_le();
            debug!("fwcfg: signature 0x{sig:x}, version {ver}");
        }

        true
    }

    fn findfile(&self, find: &str) -> Option<(u16, u32)> {
        self.select(ITEM_FILEDIR);
        let count = self.read32_be();

        for _ in 0..count {
            let size = self.read32_be();
            let item = self.read16_be();
            let _ = self.read16_be();
            let mut name: [u8; FILENAME_LEN] = [0; FILENAME_LEN];
            self.read_bytes_slice(&mut name);
            if find.as_bytes() == &name[..find.len()] {
                return Some((item, size));
            }
        }
        None
    }

    #[cfg(feature = "alloc")]
    fn readdir(&self) -> Vec<FwCfgFile> {
        let mut list = Vec::new();

        self.select(ITEM_FILEDIR);
        let count = self.read32_be();
        debug!("fwcfg: filedir: count {count}");

        for _ in 0..count {
            let size = self.read32_be();
            let item = self.read16_be();
            let _ = self.read16_be();
            let name = self.read_bytes(FILENAME_LEN);
            let entry = FwCfgFile {
                size,
                item,
                name: bytes_to_string(&name),
            };
            debug!("{entry:?}");
            list.push(entry);
        }

        list
    }

    fn select_dma(&self, item: Option<u16>) -> u32 {
        match item {
            Some(i) => ((i as u32) << 16) | DMA_CTL_SELECT,
            None => 0,
        }
    }

    fn write_dma(&self, item: u16, ptr: *const (), bytecount: usize) {
        let control = self.select_dma(Some(item)) | DMA_CTL_WRITE;
        let dmactrl = FwCfgDma {
            control: u32::swap_bytes(control),
            length: u32::swap_bytes(bytecount as u32),
            address: u64::swap_bytes(ptr.addr() as u64),
        };
        let ptr = ptr::addr_of!(dmactrl);
        self.trigger_dma(ptr.addr());
    }

    fn skip_dma(&self, item: Option<u16>, bytecount: usize) {
        let control = self.select_dma(item) | DMA_CTL_SKIP;
        let dmactrl = FwCfgDma {
            control: u32::swap_bytes(control),
            length: u32::swap_bytes(bytecount as u32),
            address: 0,
        };
        let ptr = ptr::addr_of!(dmactrl);
        self.trigger_dma(ptr.addr());
    }

    fn read_dma(&self, item: Option<u16>, ptr: *mut (), bytecount: usize) {
        let control = self.select_dma(item) | DMA_CTL_READ;
        let dmactrl = FwCfgDma {
            control: u32::swap_bytes(control),
            length: u32::swap_bytes(bytecount as u32),
            address: u64::swap_bytes(ptr.addr() as u64),
        };
        let ptr = ptr::addr_of!(dmactrl);
        self.trigger_dma(ptr.addr());
    }

    #[cfg(feature = "alloc")]
    fn load_file(&self, name: &str) -> Option<Vec<u8>> {
        let (item, size) = self.findfile(name)?;

        let (value, unit) = pretty_usize(size as usize);
        debug!("fwcfg: {name} is item {item}, size {value}{unit}");

        let mut data = vec![0; size as usize];
        self.read_dma(Some(item), data.as_mut_ptr() as *mut (), size as usize);

        Some(data)
    }
}

#[cfg(feature = "alloc")]
fn bytes_to_string(data: &[u8]) -> String {
    let mut s = String::new();
    for b in data {
        if *b == 0 {
            break;
        }
        s.push(*b as char);
    }
    s
}
