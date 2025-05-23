//!
//! scan pci bus
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::{hobs, qemu};
use virtfw_libhw::pci::PciHost;
use virtfw_libhw::pci_ecam::PciHostEcam;
#[cfg(target_arch = "x86_64")]
use virtfw_libhw::pci_x86::PciHostX86;

fn scan_one_pci_bus<H: PciHost>(pcihost: &H, bus: u8, indent: usize) {
    for slot in 0..32 {
        for func in 0..8 {
            if let Some(dev) = pcihost.try_device(bus, slot, func) {
                println!("{:i$}{}", "", dev, i = indent);

                if pcihost.is_pci_bridge(&dev) {
                    println!(
                        "{:i$}brctl: {}",
                        "",
                        pcihost.bridge_ctl(&dev),
                        i = indent + 2
                    );
                    if let Some(range) = pcihost.bridge_io(&dev) {
                        println!("{:i$}{}", "", range, i = indent + 2);
                    }
                    if let Some(range) = pcihost.bridge_mem(&dev) {
                        println!("{:i$}{}", "", range, i = indent + 2)
                    }
                    if let Some(range) = pcihost.bridge_prefmem(&dev) {
                        println!("{:i$}{}", "", range, i = indent + 2)
                    }
                    let sec = pcihost.bridge_secondary(&dev);
                    scan_one_pci_bus(pcihost, sec, indent + 4);
                }

                if pcihost.is_device(&dev) {
                    if pcihost.is_vga(&dev) {
                        println!("{:i$}cmd: {}", "", pcihost.command(&dev), i = indent + 2);
                    }
                    for b in 0..6 {
                        if let Some(range) = pcihost.bar(&dev, b) {
                            println!("{:i$}bar {}: {}", "", b, range, i = indent + 2)
                        }
                    }
                }

                if func == 0 && !pcihost.is_mfd(&dev) {
                    break;
                }
            } else if func == 0 {
                break;
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn scan_pci_bus_x86() {
    let Some(pcihost) = PciHostX86::new() else {
        println!("no x86 pci host bridge");
        return;
    };

    let hostbr = pcihost.try_device(0, 0, 0).unwrap();
    if hostbr.vendor == 0x8086 && hostbr.device == 0x29c0 {
        // q35
        let lower = pcihost.readl(&hostbr.addr, 0x60) as u64;
        let upper = pcihost.readl(&hostbr.addr, 0x64) as u64;
        if (lower & 1) == 1 {
            // enabled
            let addr = (upper << 32) | (lower & !1);
            println!("q35 ecam at 0x{:x}", addr);
            let Some(pciehost) = PciHostEcam::new(addr) else {
                println!("no pcie host bridge");
                return;
            };
            println!("scan ecam pcie bus");
            scan_one_pci_bus(&pciehost, 0, 2);
            return;
        }
    }

    println!("scan x86 pci bus");
    scan_one_pci_bus(&pcihost, 0, 2);
}

#[cfg(feature = "fdt")]
fn scan_pci_bus_fdt() {
    let Some(hl) = hobs::HobList::new() else {
        println!("failed to get hob list");
        return;
    };

    let mut iter = hl.filter_map(|h| match h {
        hobs::Hob::Fdt(data) => Some(data),
        _ => None,
    });
    let Some(fdt_addr) = iter.next() else {
        println!("no fdt hob found");
        return;
    };

    let res = unsafe { fdt::Fdt::from_ptr(fdt_addr as *const u8) };
    if let Err(e) = res {
        println!("fdt parse error: {e}");
        return;
    };
    let fdt = res.unwrap();

    let Some(ecam) = fdt.find_compatible(&["pci-host-ecam-generic"]) else {
        println!("fdt: no pcie host");
        return;
    };

    let range = ecam.reg().unwrap().next().unwrap();
    let addr = range.starting_address as u64;
    println!("pcie ecam at 0x{:x}", addr);
    let Some(pciehost) = PciHostEcam::new(addr) else {
        println!("pcie host bridge probe failed");
        return;
    };

    println!("scan ecam pcie bus");
    scan_one_pci_bus(&pciehost, 0, 2);
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    #[cfg(target_arch = "x86_64")]
    scan_pci_bus_x86();

    #[cfg(feature = "fdt")]
    scan_pci_bus_fdt();

    #[cfg(not(target_arch = "x86_64"))]
    println!("only x86 is supported right now");

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
