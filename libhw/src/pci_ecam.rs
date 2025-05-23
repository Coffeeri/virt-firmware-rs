use bitfield_struct::bitfield;

#[cfg(target_arch = "aarch64")]
use crate::mmio_arm::{readb, readl, readw, writeb, writel, writew};
#[cfg(target_arch = "x86_64")]
use crate::mmio_x86::{readb, readl, readw, writeb, writel, writew};

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
        readl(self.ecam + self.offset(dev, addr))
    }

    fn readw(&self, dev: &PciDevAddr, addr: u8) -> u16 {
        readw(self.ecam + self.offset(dev, addr))
    }

    fn readb(&self, dev: &PciDevAddr, addr: u8) -> u8 {
        readb(self.ecam + self.offset(dev, addr))
    }

    fn writel(&self, dev: &PciDevAddr, addr: u8, value: u32) {
        writel(self.ecam + self.offset(dev, addr), value);
    }

    fn writew(&self, dev: &PciDevAddr, addr: u8, value: u16) {
        writew(self.ecam + self.offset(dev, addr), value);
    }

    fn writeb(&self, dev: &PciDevAddr, addr: u8, value: u8) {
        writeb(self.ecam + self.offset(dev, addr), value);
    }
}
