//! setup qemu acpi devices, support vm poweroff aka qemu exit
#![cfg(target_arch = "x86_64")]

use crate::ioport::outw;
use crate::pci::x86::PciHostX86;
use crate::pci::PciHost;

pub struct QemuAcpi {
    pm_base: u16,
}

impl QemuAcpi {
    pub fn try_new() -> Option<QemuAcpi> {
        let acpi = QemuAcpi { pm_base: 0x600 };
        let pcihost = PciHostX86::new()?;
        let hostbr = pcihost.try_device(0, 0, 0)?;
        if hostbr.vendor == 0x8086 && hostbr.device == 0x1237 {
            acpi.init_pc(&pcihost);
            return Some(acpi);
        }
        if hostbr.vendor == 0x8086 && hostbr.device == 0x29c0 {
            acpi.init_q35(&pcihost);
            return Some(acpi);
        }
        None
    }

    fn init_pc(&self, pcihost: &PciHostX86) {
        let acpidev = pcihost.try_device(0, 1, 3).unwrap();
        // PIIX_PMBASE
        pcihost.writew(&acpidev.addr, 0x40, self.pm_base | 0x01);
        // PIIX_PMREGMISC
        pcihost.writeb(&acpidev.addr, 0x80, 0x01);
    }

    fn init_q35(&self, pcihost: &PciHostX86) {
        let smbusdev = pcihost.try_device(0, 0x1f, 0).unwrap();
        // ICH9_LPC_PMBASE
        pcihost.writel(&smbusdev.addr, 0x40, self.pm_base as u32 | 0x01);
        // ICH9_LPC_ACPI_CTRL
        pcihost.writeb(&smbusdev.addr, 0x44, 0x80);
    }

    pub fn poweroff(&self) {
        outw(self.pm_base + 4, 0x2000);
    }
}
