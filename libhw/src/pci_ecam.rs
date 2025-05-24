use bitfield_struct::bitfield;

use crate::mmio;
use crate::pci::{PciDevAddr, PciHost};

#[bitfield(u64)]
pub struct PciOffset {
    #[bits(12)]
    addr: u16,
    #[bits(3)]
    func: u8,
    #[bits(5)]
    slot: u8,
    #[bits(8)]
    bus: u8,
    #[bits(36)]
    _pad: u64,
}

pub struct PciHostEcam {
    ecam: u64,
}

impl PciHostEcam {
    pub fn new(ecam: u64) -> Option<Self> {
        let pcihost = Self { ecam };
        let _ = pcihost.try_device(0, 0, 0)?;
        Some(pcihost)
    }

    fn offset(&self, dev: &PciDevAddr, addr: u8) -> u64 {
        let offset = PciOffset::new()
            .with_bus(dev.bus)
            .with_slot(dev.slot)
            .with_func(dev.func)
            .with_addr(addr as u16);
        offset.into()
    }
}

impl PciHost for PciHostEcam {
    fn readl(&self, dev: &PciDevAddr, addr: u8) -> u32 {
        mmio::readl(self.ecam + self.offset(dev, addr))
    }

    fn readw(&self, dev: &PciDevAddr, addr: u8) -> u16 {
        mmio::readw(self.ecam + self.offset(dev, addr))
    }

    fn readb(&self, dev: &PciDevAddr, addr: u8) -> u8 {
        mmio::readb(self.ecam + self.offset(dev, addr))
    }

    fn writel(&self, dev: &PciDevAddr, addr: u8, value: u32) {
        mmio::writel(self.ecam + self.offset(dev, addr), value);
    }

    fn writew(&self, dev: &PciDevAddr, addr: u8, value: u16) {
        mmio::writew(self.ecam + self.offset(dev, addr), value);
    }

    fn writeb(&self, dev: &PciDevAddr, addr: u8, value: u8) {
        mmio::writeb(self.ecam + self.offset(dev, addr), value);
    }
}
