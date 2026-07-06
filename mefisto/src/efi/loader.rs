extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use log::{debug, info};

use virtfw_libefi::hob::igvmdata::{EfiIgvmDataHob, EfiIgvmDataType};
use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::{
    FwCfg, ITEM_CMDLINE_DATA, ITEM_CMDLINE_SIZE, ITEM_INITRD_DATA, ITEM_INITRD_SIZE,
};
use virtfw_libhw::size::*;

pub struct FwCfgLoader<'f> {
    fwcfg: &'f FwCfgX86,
    kernel_size: u32,
    kernel_item: u16,
    initrd_size: u32,
    cmdline_size: u32,
}

impl<'f> FwCfgLoader<'f> {
    pub fn new(fwcfg: &'f FwCfgX86) -> &'f Self {
        let (kernel_item, kernel_size) = fwcfg.findfile("etc/boot/kernel").unwrap_or((0, 0));
        if kernel_size != 0 {
            let (value, unit) = pretty_usize(kernel_size as usize);
            info!("fwcfg: etc/boot/kernel is item {kernel_item}, size {value}{unit}");
        }

        fwcfg.select(ITEM_INITRD_SIZE);
        let initrd_size = fwcfg.read32_le();
        if initrd_size != 0 {
            let (value, unit) = pretty_usize(initrd_size as usize);
            info!("fwcfg: initrd size {value}{unit}");
        }

        fwcfg.select(ITEM_CMDLINE_SIZE);
        let cmdline_size = fwcfg.read32_le();
        if cmdline_size != 0 {
            let (value, unit) = pretty_usize(cmdline_size as usize);
            info!("fwcfg: cmdline size {value}{unit}");
        }

        Box::leak(Box::new(Self {
            fwcfg,
            kernel_item,
            kernel_size,
            initrd_size,
            cmdline_size,
        }))
    }

    pub fn kernel_loader(&self) -> Option<&FwCfgFileLoader<'_>> {
        if self.kernel_size == 0 {
            return None;
        }
        Some(FwCfgFileLoader::new(
            self.fwcfg,
            self.kernel_size,
            self.kernel_item,
        ))
    }

    pub fn initrd_loader(&self) -> Option<&FwCfgFileLoader<'_>> {
        if self.initrd_size == 0 {
            return None;
        }
        Some(FwCfgFileLoader::new(
            self.fwcfg,
            self.initrd_size,
            ITEM_INITRD_DATA,
        ))
    }

    pub fn cmdline(&self) -> Vec<u16> {
        let mut cmdline = Vec::with_capacity(self.cmdline_size as usize);
        self.fwcfg.select(ITEM_CMDLINE_DATA);
        for _ in 0..self.cmdline_size {
            let c = self.fwcfg.read8();
            if c == 0 {
                break;
            };
            cmdline.push(c as u16);
        }
        cmdline
    }
}

pub struct FwCfgFileLoader<'fl> {
    fwcfg: &'fl FwCfgX86,
    size: u32,
    item: u16,
}

impl<'fl> FwCfgFileLoader<'fl> {
    pub fn new(fwcfg: &'fl FwCfgX86, size: u32, item: u16) -> &'fl Self {
        Box::leak(Box::new(Self { fwcfg, size, item }))
    }

    fn load_dma(&self, loadbuffer: *mut (), offset: usize, size: usize) {
        if offset > 0 {
            self.fwcfg.skip_dma(Some(self.item), offset);
            self.fwcfg.read_dma(None, loadbuffer, size);
        } else {
            self.fwcfg.read_dma(Some(self.item), loadbuffer, size);
        }
    }
}

impl efiloader::FileLoader for &FwCfgFileLoader<'_> {
    fn get_size(&self) -> usize {
        self.size as usize
    }

    fn load_file<'fl>(&self, loadbuffer: &'fl mut [MaybeUninit<u8>]) -> Result<&'fl [u8], &str> {
        debug!("load file");
        let ptr = loadbuffer.as_mut_ptr() as *mut ();
        self.load_dma(ptr, 0, self.size as usize);
        let ret = unsafe {
            core::slice::from_raw_parts(loadbuffer.as_ptr() as *const _, loadbuffer.len())
        };
        Ok(ret)
    }

    unsafe fn load_range<'fl>(
        &self,
        loadbuffer: *mut (),
        offset: usize,
        size: usize,
    ) -> Result<(), &str> {
        debug!("load range: 0x{offset:x} +0x{size:x} to {:?}", loadbuffer);
        self.load_dma(loadbuffer, offset, size);
        Ok(())
    }
}

const DATA_HOBS_MAX: usize = PAGE_4K / size_of::<EfiIgvmDataHob>();

#[link_section = ".igvm.datahobs"]
static DATA_HOBS: MaybeUninit<[EfiIgvmDataHob; DATA_HOBS_MAX]> = MaybeUninit::uninit();

pub struct IgvmFileLoader {
    hob: EfiIgvmDataHob,
    buf: &'static [u8],
}

impl IgvmFileLoader {
    pub fn new(data_type: EfiIgvmDataType) -> Option<Self> {
        let hobs = unsafe { DATA_HOBS.assume_init_ref() };
        for hob in hobs {
            if !hob.is_valid() {
                return None;
            }
            if hob.data_type != data_type as u32 {
                continue;
            }
            let loader = Self {
                hob: hob.clone(),
                buf: unsafe {
                    core::slice::from_raw_parts(hob.address as *const _, hob.length as usize)
                },
            };
            return Some(loader);
        }
        None
    }
}

impl efiloader::FileLoader for IgvmFileLoader {
    fn get_size(&self) -> usize {
        self.hob.length as usize
    }

    fn load_file<'fl>(&self, loadbuffer: &'fl mut [MaybeUninit<u8>]) -> Result<&'fl [u8], &str> {
        if loadbuffer.len() < self.buf.len() {
            return Err("size mismatch");
        }
        let buf: &mut [u8] = unsafe { loadbuffer.assume_init_mut() };
        buf.copy_from_slice(self.buf);
        Ok(buf)
    }

    unsafe fn load_range(
        &self,
        loadbuffer: *mut (),
        offset: usize,
        size: usize,
    ) -> Result<(), &str> {
        if (offset > self.hob.length as usize) || (offset + size > self.hob.length as usize) {
            return Err("size mismatch");
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(loadbuffer as *mut _, size) };
        dst.copy_from_slice(&self.buf[offset..offset + size]);
        Ok(())
    }
}
