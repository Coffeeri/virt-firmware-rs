use core::mem::MaybeUninit;
use log::{debug, info};

use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::FwCfg;

pub struct FwCfgLoader<'f> {
    fwcfg: &'f FwCfgX86,
    item: u16,
    size: u32,
}

impl<'f> FwCfgLoader<'f> {
    pub fn new(fwcfg: &'f FwCfgX86) -> Option<Self> {
        if let Some((item, size)) = fwcfg.findfile("etc/boot/kernel") {
            info!("fwcfg: etc/boot/kernel is item {item}, size {size}");
            Some(Self { fwcfg, item, size })
        } else {
            info!("fwcfg: etc/boot/kernel not found");
            None
        }
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

impl efiloader::FileLoader for FwCfgLoader<'_> {
    fn get_size(&self) -> usize {
        self.size as usize
    }

    fn load_file<'a>(&self, loadbuffer: &'a mut [MaybeUninit<u8>]) -> Result<&'a [u8], &str> {
        debug!("load file");
        let ptr = loadbuffer.as_mut_ptr() as *mut ();
        self.load_dma(ptr, 0, self.size as usize);
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
        debug!("load range: {offset} +{size}");
        self.load_dma(loadbuffer, offset, size);
        Ok(())
    }
}
