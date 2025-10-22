//! pci config space data structures and access functions
pub mod ecam;
pub mod x86;

use bitfield_struct::bitfield;
use core::fmt;

pub const PCI_CFG_VENDOR: u8 = 0x00;
pub const PCI_CFG_DEVICE: u8 = 0x02;
pub const PCI_CFG_COMMAND: u8 = 0x04;
pub const PCI_CFG_STATUS: u8 = 0x06;
pub const PCI_CFG_REVISION: u8 = 0x08;
pub const PCI_CFG_CLASS_PROG: u8 = 0x09;
pub const PCI_CFG_CLASS_DEVICE: u8 = 0x0a;
pub const PCI_CFG_CACHE_LINE_SIZE: u8 = 0x0c;
pub const PCI_CFG_LATENCY_TIMER: u8 = 0x0d;
pub const PCI_CFG_HEADER_TYPE: u8 = 0x0e;
pub const PCI_CFG_BIST: u8 = 0x0f;

// pci bridge
pub const PCI_CFG_PRIMARY_BUS: u8 = 0x18;
pub const PCI_CFG_SECONDARY_BUS: u8 = 0x19;
pub const PCI_CFG_SUBORDINATE_BUS: u8 = 0x1a;
pub const PCI_CFG_IO_BASE: u8 = 0x1c;
pub const PCI_CFG_IO_LIMIT: u8 = 0x1d;
pub const PCI_CFG_MEM_BASE: u8 = 0x20;
pub const PCI_CFG_MEM_LIMIT: u8 = 0x22;
pub const PCI_CFG_PREFMEM_BASE: u8 = 0x24;
pub const PCI_CFG_PREFMEM_LIMIT: u8 = 0x26;
pub const PCI_CFG_PREFMEM_BASE_U32: u8 = 0x28;
pub const PCI_CFG_PREFMEM_LIMIT_U32: u8 = 0x2c;
pub const PCI_CFG_BRIDGE_CTRL: u8 = 0x3e;

pub const PCI_CFG_SUB_VENDOR: u8 = 0x2c;
pub const PCI_CFG_SUB_DEVICE: u8 = 0x2e;

#[bitfield(u16)]
pub struct PciCommand {
    pub io: bool,
    pub memory: bool,
    pub master: bool,
    pub special: bool,
    pub invalidate: bool,
    pub vga_palette: bool,
    pub parity: bool,
    pub wait: bool,
    pub serr: bool,
    pub fast_back: bool,
    pub intx_disable: bool,
    #[bits(5)]
    _pad: u8,
}

impl fmt::Display for PciCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "io{} memory{} master{} special{} inval{} vga-snoop{} parity{} wait{} serr{} fast-back{} no-intx{}",
            if self.io() { "+" } else { "-" },
            if self.memory() { "+" } else { "-" },
            if self.master() { "+" } else { "-" },
            if self.special() { "+" } else { "-" },
            if self.invalidate() { "+" } else { "-" },
            if self.vga_palette() { "+" } else { "-" },
            if self.parity() { "+" } else { "-" },
            if self.wait() { "+" } else { "-" },
            if self.serr() { "+" } else { "-" },
            if self.fast_back() { "+" } else { "-" },
            if self.intx_disable() { "+" } else { "-" },
        )
    }
}

#[bitfield(u8)]
pub struct PciHeaderType {
    #[bits(7)]
    pub value: u8,
    pub mfd: bool,
}

#[bitfield(u8)]
pub struct PciBridgeCtrl {
    pub parity: bool,
    pub serr: bool,
    pub isa: bool,
    pub vga: bool,
    _pad: bool,
    pub master_abort: bool,
    pub bus_reset: bool,
    pub fast_back: bool,
}

impl fmt::Display for PciBridgeCtrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "parity{} serr{} isa{} vga{} abort{} bus-reset{} fast-back{}",
            if self.parity() { "+" } else { "-" },
            if self.serr() { "+" } else { "-" },
            if self.isa() { "+" } else { "-" },
            if self.vga() { "+" } else { "-" },
            if self.master_abort() { "+" } else { "-" },
            if self.bus_reset() { "+" } else { "-" },
            if self.fast_back() { "+" } else { "-" },
        )
    }
}

#[derive(Debug)]
pub struct PciAddrRange {
    pub name: &'static str,
    pub base: u64,
    pub limit: u64,
}

impl fmt::Display for PciAddrRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.base > 0 {
            write!(f, "{}: {:x}-{:x}", self.name, self.base, self.limit)
        } else {
            write!(f, "{}: <size=0x{:x}>", self.name, self.limit + 1)
        }
    }
}

#[derive(Debug)]
pub struct PciDevAddr {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
}

impl fmt::Display for PciDevAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x}:{:02x}.{:x}", self.bus, self.slot, self.func)
    }
}

pub struct PciDevice {
    pub addr: PciDevAddr,
    pub vendor: u16,
    pub device: u16,
    pub class: u16,
}

impl fmt::Display for PciDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}  {:04x}:{:04x}  [{:04x} {}]  {}",
            self.addr,
            self.vendor,
            self.device,
            self.class,
            self.class_name(),
            self.device_name(),
        )
    }
}

impl PciDevice {
    pub fn device_name(&self) -> &str {
        match (self.vendor, self.device) {
            // 'pc' machine type
            (0x8086, 0x1237) => "intel i440fx",
            (0x8086, 0x7000) => "intel piix4 lpc",
            (0x8086, 0x7010) => "intel piix4 ide",
            (0x8086, 0x7020) => "intel piix4 uhci",
            (0x8086, 0x7113) => "intel piix4 acpi",

            // 'q35' machine type
            (0x8086, 0x29c0) => "intel q35",
            (0x8086, 0x2918) => "intel ich9 lpc",
            (0x8086, 0x2922) => "intel ich9 ahci",
            (0x8086, 0x2930) => "intel ich9 smbus",

            (0x8086, 0x2934) => "intel ich9 uhci1",
            (0x8086, 0x2935) => "intel ich9 uhci2",
            (0x8086, 0x2936) => "intel ich9 uhci3",
            (0x8086, 0x293a) => "intel ich9 ehci",

            // intel device
            (0x8086, 0x100e) => "intel e1000",
            (0x8086, 0x10d3) => "intel e1000e",
            (0x8086, _) => "intel unknown",

            // virtio devices (vendor 0x1af4)
            (0x1af4, 0x1000) => "virtio net",
            (0x1af4, 0x1001) => "virtio blk",
            (0x1af4, 0x1003) => "virtio console",
            (0x1af4, 0x1004) => "virtio scsi",
            (0x1af4, 0x1005) => "virtio rng",
            (0x1af4, 0x1012) => "virtio vsock",

            (0x1af4, 0x1041) => "virtio net 1.0",
            (0x1af4, 0x1042) => "virtio blk 1.0",
            (0x1af4, 0x1043) => "virtio console 1.0",
            (0x1af4, 0x1044) => "virtio rng 1.0",
            (0x1af4, 0x1048) => "virtio scsi 1.0",
            (0x1af4, 0x1050) => "virtio gpu",
            (0x1af4, 0x1052) => "virtio input",
            (0x1af4, 0x1053) => "virtio vsock 1.0",

            (0x1af4, _) => "virtio unknown",

            // qemu devices (vendor 0x1b36)
            (0x1b36, 0x0001) => "qemu pci-pci bridge",
            (0x1b36, 0x0007) => "qemu sdhci",
            (0x1b36, 0x0008) => "qemu pcie host bridge",
            (0x1b36, 0x000c) => "qemu pcie root port",
            (0x1b36, 0x000d) => "qemu xhci",
            (0x1b36, 0x000e) => "qemu pcie-pci bridge",
            (0x1b36, 0x0010) => "qemu nvme",
            (0x1b36, _) => "qemu unknown",

            // qemu devices (other)
            (0x1234, 0x1111) => "qemu stdvga",

            // catch all
            (_, _) => "unknown",
        }
    }

    pub fn class_name(&self) -> &str {
        match self.class {
            0x0100 => "scsi",
            0x0101 => "ide",
            0x0105 => "ata",
            0x0106 => "sata",
            0x0180 => "storage",

            0x0200 => "ethernet",
            0x0280 => "network",

            0x0300 => "vga",
            0x0380 => "display",

            0x0600 => "host bridge",
            0x0601 => "isa bridge",
            0x0604 => "pci bridge",

            0x0c03 => "usb",
            0x0c05 => "smbus",

            _ => "unknown",
        }
    }
}

pub trait PciHost {
    fn readl(&self, dev: &PciDevAddr, addr: u8) -> u32;
    fn readw(&self, dev: &PciDevAddr, addr: u8) -> u16;
    fn readb(&self, dev: &PciDevAddr, addr: u8) -> u8;

    fn writel(&self, dev: &PciDevAddr, addr: u8, value: u32);
    fn writew(&self, dev: &PciDevAddr, addr: u8, value: u16);
    fn writeb(&self, dev: &PciDevAddr, addr: u8, value: u8);

    fn try_device(&self, bus: u8, slot: u8, func: u8) -> Option<PciDevice> {
        let addr = PciDevAddr { bus, slot, func };
        let vendor = self.readw(&addr, PCI_CFG_VENDOR);
        if (vendor == 0x0000) || (vendor == 0xffff) {
            return None;
        }
        let device = self.readw(&addr, PCI_CFG_DEVICE);
        let class = self.readw(&addr, PCI_CFG_CLASS_DEVICE);
        let pcidev = PciDevice {
            addr,
            vendor,
            device,
            class,
        };
        Some(pcidev)
    }

    fn command(&self, dev: &PciDevice) -> PciCommand {
        PciCommand::from(self.readw(&dev.addr, PCI_CFG_COMMAND))
    }

    fn revision(&self, dev: &PciDevice) -> u8 {
        self.readb(&dev.addr, PCI_CFG_DEVICE)
    }

    fn header_type(&self, dev: &PciDevice) -> PciHeaderType {
        PciHeaderType::from(self.readb(&dev.addr, PCI_CFG_HEADER_TYPE))
    }

    fn is_vga(&self, dev: &PciDevice) -> bool {
        dev.class == 0x0300
    }

    fn is_mfd(&self, dev: &PciDevice) -> bool {
        let h = self.header_type(dev);
        h.mfd()
    }

    fn is_device(&self, dev: &PciDevice) -> bool {
        if self.header_type(dev).value() != 0 {
            return false;
        }
        true
    }

    fn bar(&self, dev: &PciDevice, nr: u8) -> Option<PciAddrRange> {
        if nr > 5 {
            return None;
        }
        let mut base = self.readl(&dev.addr, 0x10 + nr * 4) as u64;
        if nr > 0 {
            let prevbase = self.readl(&dev.addr, 0x10 + (nr - 1) * 4);
            if (prevbase & 6) == 4 {
                // previous bar is 64-bit
                return None;
            }
        }

        self.writel(&dev.addr, 0x10 + nr * 4, !0);
        let mut size = self.readl(&dev.addr, 0x10 + nr * 4) as u64;
        self.writel(&dev.addr, 0x10 + nr * 4, base as u32);

        if base == 0 {
            None
        } else if (base & 1) == 1 {
            // io
            base &= !0x03;
            size &= !0x03;
            size = !size & 0xffffffff;
            Some(PciAddrRange {
                name: "io",
                base,
                limit: base + size,
            })
        } else if (base & 6) == 0 {
            // 32 bit mem
            let prefmem = (base & 0x08) != 0;
            base &= !0x0f;
            size &= !0x0f;
            size = !size & 0xffffffff;
            Some(PciAddrRange {
                name: if prefmem { "prefmem32" } else { "mem32" },
                base,
                limit: base + size,
            })
        } else if (base & 6) == 4 {
            // 64 bit mem
            let prefmem = (base & 0x08) != 0;
            base &= !0x0f;
            size &= !0x0f;

            let nextbase = self.readl(&dev.addr, 0x10 + (nr + 1) * 4) as u64;
            self.writel(&dev.addr, 0x10 + (nr + 1) * 4, !0);
            let nextsize = self.readl(&dev.addr, 0x10 + (nr + 1) * 4) as u64;
            self.writel(&dev.addr, 0x10 + (nr + 1) * 4, nextbase as u32);

            base += nextbase << 32;
            size += nextsize << 32;
            size = !size;

            Some(PciAddrRange {
                name: if prefmem { "prefmem64" } else { "mem64" },
                base,
                limit: base + size,
            })
        } else {
            Some(PciAddrRange {
                name: "FIXME",
                base,
                limit: 0,
            })
        }
    }

    fn is_pci_bridge(&self, dev: &PciDevice) -> bool {
        if dev.class != 0x0604 {
            return false;
        }
        if self.header_type(dev).value() != 1 {
            return false;
        }
        true
    }

    fn bridge_ctl(&self, dev: &PciDevice) -> PciBridgeCtrl {
        PciBridgeCtrl::from(self.readb(&dev.addr, PCI_CFG_BRIDGE_CTRL))
    }

    fn bridge_secondary(&self, dev: &PciDevice) -> u8 {
        self.readb(&dev.addr, PCI_CFG_SECONDARY_BUS)
    }

    fn bridge_subordinate(&self, dev: &PciDevice) -> u8 {
        self.readb(&dev.addr, PCI_CFG_SUBORDINATE_BUS)
    }

    fn bridge_io(&self, dev: &PciDevice) -> Option<PciAddrRange> {
        let basereg = self.readb(&dev.addr, PCI_CFG_IO_BASE) as u64;
        let limitreg = self.readb(&dev.addr, PCI_CFG_IO_LIMIT) as u64;
        if limitreg != 0 {
            let base = (basereg & 0xf0) << 8;
            let limit = ((limitreg & 0xf0) << 8) + 0xfff;
            Some(PciAddrRange {
                name: "io",
                base,
                limit,
            })
        } else {
            None
        }
    }

    fn bridge_mem(&self, dev: &PciDevice) -> Option<PciAddrRange> {
        let basereg = self.readw(&dev.addr, PCI_CFG_MEM_BASE) as u64;
        let limitreg = self.readw(&dev.addr, PCI_CFG_MEM_LIMIT) as u64;
        if limitreg != 0 {
            let base = (basereg & 0xfff0) << 16;
            let limit = ((limitreg & 0xfff0) << 16) + 0xfffff;
            Some(PciAddrRange {
                name: "mem",
                base,
                limit,
            })
        } else {
            None
        }
    }

    fn bridge_prefmem(&self, dev: &PciDevice) -> Option<PciAddrRange> {
        let basereg = self.readw(&dev.addr, PCI_CFG_PREFMEM_BASE) as u64;
        let limitreg = self.readw(&dev.addr, PCI_CFG_PREFMEM_LIMIT) as u64;
        let basereg_u32 = self.readl(&dev.addr, PCI_CFG_PREFMEM_BASE_U32) as u64;
        let limitreg_u32 = self.readl(&dev.addr, PCI_CFG_PREFMEM_LIMIT_U32) as u64;

        let mut base = (basereg & 0xfff0) << 16;
        let mut limit = (limitreg & 0xfff0) << 16;
        base |= basereg_u32 << 32;
        limit |= limitreg_u32 << 32;

        if limit != 0 {
            limit += 0xfffff;
            Some(PciAddrRange {
                name: "prefmem64",
                base,
                limit,
            })
        } else {
            None
        }
    }

    fn set_bridge_primary(&self, dev: &PciDevice, nr: u8) {
        self.writeb(&dev.addr, PCI_CFG_PRIMARY_BUS, nr)
    }

    fn set_bridge_secondary(&self, dev: &PciDevice, nr: u8) {
        self.writeb(&dev.addr, PCI_CFG_SECONDARY_BUS, nr)
    }

    fn set_bridge_subordinate(&self, dev: &PciDevice, nr: u8) {
        self.writeb(&dev.addr, PCI_CFG_SUBORDINATE_BUS, nr)
    }
}
