#![cfg(target_arch = "x86_64")]
use core::ops::Range;
use log::debug;

use crate::ioport::{outb, outw};
use crate::pci::ecam::PciHostEcam;
use crate::pci::x86::PciHostX86;
use crate::pci::{PciDevice, PciHost};

const PM_BASE: u16 = 0x600;

pub enum QemuBoard {
    I440fx(QemuI440fx),
    Q35(QemuQ35),
}

impl QemuBoard {
    pub fn init_acpi(&self) {
        match self {
            QemuBoard::I440fx(board) => board.init_acpi(),
            QemuBoard::Q35(board) => board.init_acpi(),
        }
    }

    pub fn pcihost(&self) -> &dyn PciHost {
        match self {
            QemuBoard::I440fx(board) => &board.pcihost,
            QemuBoard::Q35(board) => &board.pciehost,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            QemuBoard::I440fx(_) => "qemu pc machine",
            QemuBoard::Q35(_) => "qemu q35 machine",
        }
    }

    pub fn ecam(&self) -> Option<Range<usize>> {
        match self {
            QemuBoard::I440fx(_) => None,
            QemuBoard::Q35(b) => {
                let base = b.pciehost.ecam as usize;
                Some(base..base + 0x1000_0000)
            }
        }
    }

    pub fn poweroff(&self) {
        outw(PM_BASE + 4, 0x2000);
    }

    pub fn reset(&self) {
        outb(0xCF9, 0x0f);
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

    pub fn init_acpi(&self) {
        let acpidev = self.pcihost.try_device(0, 1, 3).unwrap();
        // PIIX_PMBASE
        self.pcihost.writew(&acpidev.addr, 0x40, PM_BASE | 0x01);
        // PIIX_PMREGMISC
        self.pcihost.writeb(&acpidev.addr, 0x80, 0x01);
    }
}

pub struct QemuQ35 {
    pciehost: PciHostEcam,
}

impl QemuQ35 {
    pub fn try_new() -> Option<Self> {
        let pcihost = PciHostX86::new()?;
        let hostbr = pcihost.try_device(0, 0, 0)?;
        if hostbr.vendor != 0x8086 || hostbr.device != 0x29c0 {
            return None;
        }
        let pciehost = Self::pcie_init(&pcihost, &hostbr)?;
        Some(Self { pciehost })
    }

    pub fn pcie_init(pcihost: &PciHostX86, hostbr: &PciDevice) -> Option<PciHostEcam> {
        // q35
        let lower = pcihost.readl(&hostbr.addr, 0x60) as u64;
        let upper = pcihost.readl(&hostbr.addr, 0x64) as u64;
        let addr = if (lower & 1) == 1 {
            // already enabled
            let addr = (upper << 32) | (lower & !1);
            debug!("found q35 ecam at 0x{:x}", addr);
            addr
        } else {
            let upper = 0_u64;
            let lower = 0xe000_0000_u64 | 1;
            let addr = (upper << 32) | (lower & !1);
            pcihost.writel(&hostbr.addr, 0x64, upper as u32);
            pcihost.writel(&hostbr.addr, 0x60, lower as u32);
            debug!("mapped q35 ecam to 0x{:x}", addr);
            addr
        };
        PciHostEcam::new(addr)
    }

    pub fn init_acpi(&self) {
        let smbusdev = self.pciehost.try_device(0, 0x1f, 0).unwrap();
        // ICH9_LPC_PMBASE
        let addr = PM_BASE as u32;
        self.pciehost.writel(&smbusdev.addr, 0x40, addr | 0x01);
        // ICH9_LPC_ACPI_CTRL
        self.pciehost.writeb(&smbusdev.addr, 0x44, 0x80);
    }
}

pub fn qemu_probe() -> Option<QemuBoard> {
    if let Some(q35) = QemuQ35::try_new() {
        return Some(QemuBoard::Q35(q35));
    }
    if let Some(pc) = QemuI440fx::try_new() {
        return Some(QemuBoard::I440fx(pc));
    }
    None
}

pub fn qemu_try_poweroff() {
    if let Some(board) = qemu_probe() {
        board.init_acpi();
        board.poweroff();
    }
}
