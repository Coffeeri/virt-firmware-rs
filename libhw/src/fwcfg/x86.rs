#![cfg(target_arch = "x86_64")]

use crate::fwcfg::FwCfg;
use crate::ioport;

const IOPORT_ITEM: u16 = 0x510;
const IOPORT_DATA: u16 = 0x511;
const IOPORT_DMA: u16 = 0x514;

pub struct FwCfgX86 {}

impl FwCfgX86 {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for FwCfgX86 {
    fn default() -> Self {
        Self::new()
    }
}

impl FwCfg for FwCfgX86 {
    fn select(&self, item: u16) {
        ioport::outw(IOPORT_ITEM, item);
    }

    fn read8(&self) -> u8 {
        ioport::inb(IOPORT_DATA)
    }

    fn trigger_dma(&self, addr: usize) {
        let addr_hi: u32 = (addr >> 32) as u32;
        let addr_lo: u32 = (addr & 0xffffffff) as u32;
        ioport::outl(IOPORT_DMA, u32::swap_bytes(addr_hi));
        ioport::outl(IOPORT_DMA + 4, u32::swap_bytes(addr_lo));
    }
}
