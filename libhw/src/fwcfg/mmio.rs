use crate::fwcfg::FwCfg;
use crate::mmio;

const MMIO_OFFSET_ITEM: u64 = 0x08;
const MMIO_OFFSET_DATA: u64 = 0x00;
const MMIO_OFFSET_DMA: u64 = 0x10;

pub struct FwCfgMmio {
    base: u64,
}

impl FwCfgMmio {
    pub const fn new(base: u64) -> Self {
        Self { base }
    }
}

impl FwCfg for FwCfgMmio {
    fn select(&self, item: u16) {
        mmio::writew(self.base + MMIO_OFFSET_ITEM, u16::swap_bytes(item));
    }

    fn read8(&self) -> u8 {
        mmio::readb(self.base + MMIO_OFFSET_DATA)
    }

    fn trigger_dma(&self, addr: usize) {
        mmio::writeq(
            self.base + MMIO_OFFSET_DMA,
            u64::swap_bytes(addr.try_into().unwrap()),
        );
    }
}
