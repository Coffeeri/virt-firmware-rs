extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use log::{debug, info, trace};

use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::{
    FwCfg, ITEM_CMDLINE_DATA, ITEM_CMDLINE_SIZE, ITEM_INITRD_DATA, ITEM_INITRD_SIZE,
};

use crate::size::*;

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
        trace!("load range: {offset} +{size}");
        self.load_dma(loadbuffer, offset, size);
        Ok(())
    }
}
