#!cfg[(target_arch = "x86_64")]

use crate::ioport::outw;
use crate::pci::PciDevice;

pub struct QemuAcpi {
    pm_base: u16,
}

impl QemuAcpi {
    pub fn try_new() -> Option<QemuAcpi> {
        let acpi = QemuAcpi { pm_base: 0x600 };
        let host = PciDevice::try_new(0, 0, 0)?;
        if host.vendor == 0x8086 && host.device == 0x1237 {
            acpi.init_pc();
            return Some(acpi);
        }
        if host.vendor == 0x8086 && host.device == 0x29c0 {
            acpi.init_q35();
            return Some(acpi);
        }
        None
    }

    fn init_pc(&self) {
        let acpidev = PciDevice::try_new(0, 1, 3).unwrap();
        // PIIX_PMBASE
        acpidev.addr.writew(0x40, self.pm_base | 0x01);
        // PIIX_PMREGMISC
        acpidev.addr.writeb(0x80, 0x01);
    }

    fn init_q35(&self) {
        let smbusdev = PciDevice::try_new(0, 0x1f, 0).unwrap();
        // ICH9_LPC_PMBASE
        smbusdev.addr.writel(0x40, self.pm_base as u32 | 0x01);
        // ICH9_LPC_ACPI_CTRL
        smbusdev.addr.writeb(0x44, 0x80);
    }

    pub fn poweroff(&self) {
        outw(self.pm_base + 4, 0x2000);
    }
}
