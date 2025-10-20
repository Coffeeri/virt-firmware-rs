extern crate alloc;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use log::{debug, info, trace};

use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::{FwCfg, ITEM_CMDLINE_DATA, ITEM_CMDLINE_SIZE};

pub struct FwCfgLoader<'f> {
    fwcfg: &'f FwCfgX86,
    kernel_size: u32,
    kernel_item: u16,
    cmdline_size: u32,
}

impl<'f> FwCfgLoader<'f> {
    pub fn new(fwcfg: &'f FwCfgX86) -> Option<Self> {
        let Some((item, size)) = fwcfg.findfile("etc/boot/kernel") else {
            info!("fwcfg: etc/boot/kernel not found");
            return None;
        };
        info!("fwcfg: etc/boot/kernel is item {item}, size {size}");

        fwcfg.select(ITEM_CMDLINE_SIZE);
        let cmdline_size = fwcfg.read32_le();
        info!("fwcfg: cmdline size {cmdline_size} bytes");

        Some(Self {
            fwcfg,
            kernel_item: item,
            kernel_size: size,
            cmdline_size,
        })
    }

    fn load_dma(&self, loadbuffer: *mut (), offset: usize, size: usize) {
        if offset > 0 {
            self.fwcfg.skip_dma(Some(self.kernel_item), offset);
            self.fwcfg.read_dma(None, loadbuffer, size);
        } else {
            self.fwcfg
                .read_dma(Some(self.kernel_item), loadbuffer, size);
        }
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

impl efiloader::FileLoader for FwCfgLoader<'_> {
    fn get_size(&self) -> usize {
        self.kernel_size as usize
    }

    fn load_file<'a>(&self, loadbuffer: &'a mut [MaybeUninit<u8>]) -> Result<&'a [u8], &str> {
        debug!("load file");
        let ptr = loadbuffer.as_mut_ptr() as *mut ();
        self.load_dma(ptr, 0, self.kernel_size as usize);
        let ret = unsafe {
            core::slice::from_raw_parts(loadbuffer.as_ptr() as *const _, loadbuffer.len())
        };
        Ok(ret)
    }

    unsafe fn load_range<'a>(
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
