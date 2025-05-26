#![cfg(target_arch = "x86_64")]

use bitfield_struct::bitfield;

use crate::ioport;
use crate::pci::{PciDevAddr, PciHost};

const IOPORT_PCI_CMD: u16 = 0x0cf8;
const IOPORT_PCI_DATA: u16 = 0x0cfc;

#[bitfield(u32)]
pub struct PciCmd {
    #[bits(8)]
    addr: u8,
    #[bits(3)]
    func: u8,
    #[bits(5)]
    slot: u8,
    #[bits(8)]
    bus: u8,
    #[bits(7)]
    _pad: u8,
    #[bits(default = true)]
    enable: bool,
}

pub struct PciHostX86 {}

impl PciHostX86 {
    pub fn new() -> Option<Self> {
        let pcihost = Self {};
        let _ = pcihost.try_device(0, 0, 0)?;
        Some(pcihost)
    }

    fn select(&self, dev: &PciDevAddr, addr: u8) {
        let cmd = PciCmd::new()
            .with_bus(dev.bus)
            .with_slot(dev.slot)
            .with_func(dev.func)
            .with_addr(addr);
        ioport::outl(IOPORT_PCI_CMD, cmd.into());
    }
}

impl PciHost for PciHostX86 {
    fn readl(&self, dev: &PciDevAddr, addr: u8) -> u32 {
        self.select(dev, addr);
        ioport::inl(IOPORT_PCI_DATA)
    }

    fn readw(&self, dev: &PciDevAddr, addr: u8) -> u16 {
        self.select(dev, addr);
        ioport::inw(IOPORT_PCI_DATA)
    }

    fn readb(&self, dev: &PciDevAddr, addr: u8) -> u8 {
        self.select(dev, addr);
        ioport::inb(IOPORT_PCI_DATA)
    }

    fn writel(&self, dev: &PciDevAddr, addr: u8, value: u32) {
        self.select(dev, addr);
        ioport::outl(IOPORT_PCI_DATA, value);
    }

    fn writew(&self, dev: &PciDevAddr, addr: u8, value: u16) {
        self.select(dev, addr);
        ioport::outw(IOPORT_PCI_DATA, value);
    }

    fn writeb(&self, dev: &PciDevAddr, addr: u8, value: u8) {
        self.select(dev, addr);
        ioport::outb(IOPORT_PCI_DATA, value);
    }
}
