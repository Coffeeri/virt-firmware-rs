#![cfg(target_arch = "x86_64")]

extern crate alloc;
use alloc::boxed::Box;

use crate::ioport::outw;
use crate::pci::x86::PciHostX86;
use crate::pci::PciHost;

const PM_BASE: u16 = 0x600;

pub trait QemuBoard {
    fn init_acpi(&self);

    fn poweroff(&self) {
        outw(PM_BASE + 4, 0x2000);
    }
}

pub struct QemuI440fx {
    pcihost: PciHostX86,
}

impl QemuI440fx {
    pub fn try_new() -> Option<Self> {
        let pcihost = PciHostX86::new()?;
        let hostbr = pcihost.try_device(0, 0, 0)?;
        if hostbr.vendor != 0x8086 || hostbr.device != 0x1237 {
            return None;
        }
        Some(Self { pcihost })
    }
}

impl QemuBoard for QemuI440fx {
    fn init_acpi(&self) {
        let acpidev = self.pcihost.try_device(0, 1, 3).unwrap();
        // PIIX_PMBASE
        self.pcihost.writew(&acpidev.addr, 0x40, PM_BASE | 0x01);
        // PIIX_PMREGMISC
        self.pcihost.writeb(&acpidev.addr, 0x80, 0x01);
    }
}

pub struct QemuQ35 {
    pcihost: PciHostX86,
}

impl QemuQ35 {
    pub fn try_new() -> Option<Self> {
        let pcihost = PciHostX86::new()?;
        let hostbr = pcihost.try_device(0, 0, 0)?;
        if hostbr.vendor != 0x8086 || hostbr.device != 0x29c0 {
            return None;
        }
        Some(Self { pcihost })
    }
}

impl QemuBoard for QemuQ35 {
    fn init_acpi(&self) {
        let smbusdev = self.pcihost.try_device(0, 0x1f, 0).unwrap();
        // ICH9_LPC_PMBASE
        let addr = PM_BASE as u32;
        self.pcihost.writel(&smbusdev.addr, 0x40, addr | 0x01);
        // ICH9_LPC_ACPI_CTRL
        self.pcihost.writeb(&smbusdev.addr, 0x44, 0x80);
    }
}

pub fn qemu_probe() -> Option<Box<dyn QemuBoard>> {
    if let Some(q35) = QemuQ35::try_new() {
        return Some(Box::new(q35));
    }
    if let Some(pc) = QemuI440fx::try_new() {
        return Some(Box::new(pc));
    }
    None
}

pub fn qemu_try_poweroff() {
    if let Some(board) = qemu_probe() {
        board.init_acpi();
        board.poweroff();
    }
}
