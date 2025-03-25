#![cfg(target_arch = "x86_64") && (target_os = "uefi")]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use log::debug;

use core::ptr;

use crate::ioport;

const IOPORT_ITEM: u16 = 0x510;
const IOPORT_DATA: u16 = 0x511;
const IOPORT_DMA: u16 = 0x514;

pub const ITEM_SIGNATURE: u16 = 0x00;
pub const ITEM_VERSION: u16 = 0x01;
pub const ITEM_FILEDIR: u16 = 0x19;

pub fn select(item: u16) {
    ioport::outw(IOPORT_ITEM, item)
}

pub fn read8() -> u8 {
    ioport::inb(IOPORT_DATA)
}

#[cfg(feature = "alloc")]
pub fn read_bytes(count: usize) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();
    for _ in 0..count {
        data.push(read8());
    }
    data
}

pub fn read_bytes_slice(dest: &mut [u8]) {
    for byte in dest {
        *byte = read8();
    }
}

pub fn read16_le() -> u16 {
    let b1 = read8() as u16;
    let b2 = read8() as u16;
    b1 | (b2 << 8)
}

pub fn read32_le() -> u32 {
    let w1 = read16_le() as u32;
    let w2 = read16_le() as u32;
    w1 | (w2 << 16)
}

pub fn read64_le() -> u64 {
    let l1 = read32_le() as u64;
    let l2 = read32_le() as u64;
    l1 | (l2 << 32)
}

pub fn read16_be() -> u16 {
    let b1 = read8() as u16;
    let b2 = read8() as u16;
    b2 | (b1 << 8)
}

pub fn read32_be() -> u32 {
    let w1 = read16_be() as u32;
    let w2 = read16_be() as u32;
    w2 | (w1 << 16)
}

pub fn read64_be() -> u64 {
    let l1 = read32_be() as u64;
    let l2 = read32_be() as u64;
    l2 | (l1 << 32)
}

pub fn detect() -> bool {
    select(ITEM_SIGNATURE);
    let sig = read32_le();
    if sig != 0x554d4551 {
        return false;
    }

    #[cfg(feature = "alloc")]
    {
        select(ITEM_VERSION);
        let ver = read32_le();
        debug!("fwcfg: signature 0x{:x}, version {}", sig, ver);
    }

    true
}

#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct FwCfgFile {
    pub size: u32,
    pub item: u16,
    pub name: String,
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

pub fn findfile(find: &str) -> Option<(u16, u32)> {
    select(ITEM_FILEDIR);
    let count = read32_be();

    for _ in 0..count {
        let size = read32_be();
        let item = read16_be();
        let _ = read16_be();
        let mut name: [u8; 56] = [0; 56];
        read_bytes_slice(&mut name);
        if find.as_bytes() == &name[..find.len()] {
            return Some((item, size));
        }
    }
    None
}

#[cfg(feature = "alloc")]
pub fn readdir() -> Vec<FwCfgFile> {
    let mut list = Vec::new();

    select(ITEM_FILEDIR);
    let count = read32_be();
    debug!("fwcfg: filedir: count {}", count);

    for _ in 0..count {
        let size = read32_be();
        let item = read16_be();
        let _ = read16_be();
        let name = read_bytes(56);
        let entry = FwCfgFile {
            size,
            item,
            name: bytes_to_string(&name),
        };
        debug!("{:?}", entry);
        list.push(entry);
    }

    list
}

#[repr(C)]
struct FwCfgDma {
    control: u32,
    length: u32,
    address: u64,
}

pub fn write_dma<T>(item: u16, ptr: *const T, bytecount: usize) {
    let control = ((item as u32) << 16) | 0x18; // select + write
    let dmactrl = FwCfgDma {
        control: u32::swap_bytes(control),
        length: u32::swap_bytes(bytecount as u32),
        address: u64::swap_bytes(ptr.addr() as u64),
    };
    let ptr = ptr::addr_of!(dmactrl);
    let ptr_hi: u32 = (ptr.addr() >> 32) as u32;
    let ptr_lo: u32 = (ptr.addr() & 0xffffffff) as u32;
    ioport::outl(IOPORT_DMA, u32::swap_bytes(ptr_hi));
    ioport::outl(IOPORT_DMA + 4, u32::swap_bytes(ptr_lo));
}
