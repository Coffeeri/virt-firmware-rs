//!
//! scan pci bus
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::misc;
use virtfw_efi_apps::pager::Pager;
use virtfw_efi_apps::qemu;
use virtfw_libhw::pci::ecam::PciHostEcam;
use virtfw_libhw::pci::PciHost;

#[cfg(target_arch = "x86_64")]
use virtfw_libhw::pci::x86::PciHostX86;

#[cfg(feature = "fdt")]
use virtfw_efi_apps::fdt;

fn scan_one_pci_bus<H: PciHost>(pager: &mut Pager, pcihost: &H, bus: u8, indent: usize) {
    for slot in 0..32 {
        for func in 0..8 {
            if let Some(dev) = pcihost.try_device(bus, slot, func) {
                println!("{:i$}{}", "", dev, i = indent);
                pager.lines(1);

                if pcihost.is_pci_bridge(&dev) {
                    println!(
                        "{:i$}brctl: {}",
                        "",
                        pcihost.bridge_ctl(&dev),
                        i = indent + 2
                    );
                    pager.lines(1);
                    if let Some(range) = pcihost.bridge_io(&dev) {
                        println!("{:i$}{}", "", range, i = indent + 2);
                        pager.lines(1);
                    }
                    if let Some(range) = pcihost.bridge_mem(&dev) {
                        println!("{:i$}{}", "", range, i = indent + 2);
                        pager.lines(1);
                    }
                    if let Some(range) = pcihost.bridge_prefmem(&dev) {
                        println!("{:i$}{}", "", range, i = indent + 2);
                        pager.lines(1);
                    }
                    let sec = pcihost.bridge_secondary(&dev);
                    scan_one_pci_bus(pager, pcihost, sec, indent + 4);
                }

                if pcihost.is_device(&dev) {
                    if pcihost.is_vga(&dev) {
                        println!("{:i$}cmd: {}", "", pcihost.command(&dev), i = indent + 2);
                        pager.lines(1);
                    }
                    for b in 0..6 {
                        if let Some(range) = pcihost.bar(&dev, b) {
                            println!("{:i$}bar {}: {}", "", b, range, i = indent + 2);
                            pager.lines(1);
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
fn scan_pci_bus_x86(pager: &mut Pager) -> bool {
    let Some(pcihost) = PciHostX86::new() else {
        println!("no x86 pci host bridge");
        pager.lines(1);
        return false;
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
            pager.lines(1);
            let Some(pciehost) = PciHostEcam::new(addr) else {
                println!("no pcie host bridge");
                pager.lines(1);
                return false;
            };
            println!("scan ecam pcie bus");
            pager.lines(1);
            scan_one_pci_bus(pager, &pciehost, 0, 2);
            return true;
        }
    }

    println!("scan x86 pci bus");
    pager.lines(1);
    scan_one_pci_bus(pager, &pcihost, 0, 2);
    true
}

#[cfg(feature = "fdt")]
fn scan_pci_bus_fdt(pager: &mut Pager) -> bool {
    let addr = match fdt::find_addr("pci-host-ecam-generic") {
        Err(e) => {
            println!("{e}");
            pager.lines(1);
            return false;
        }
        Ok(a) => a,
    };

    println!("pcie ecam at 0x{:x}", addr);
    let Some(pciehost) = PciHostEcam::new(addr) else {
        println!("pcie host bridge probe failed");
        pager.lines(1);
        return false;
    };

    println!("scan ecam pcie bus");
    pager.lines(1);
    scan_one_pci_bus(pager, &pciehost, 0, 2);
    true
}

fn scan_pci_bus(pager: &mut Pager) {
    #[cfg(target_arch = "x86_64")]
    {
        if scan_pci_bus_x86(pager) {
            return;
        }
    }

    #[cfg(feature = "fdt")]
    {
        if scan_pci_bus_fdt(pager) {
            return;
        }
    }

    println!("pci scan not supported on this platform");
    pager.lines(1);
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let mut use_pager = false;
    if let Some(cmdline) = misc::cmdline("hoblist") {
        if cmdline == "-p" || cmdline == "pager" {
            use_pager = true;
        }
    }

    let mut pager = Pager::new(use_pager);

    scan_pci_bus(&mut pager);

    pager.end();
    qemu::shutdown_if_direct_kernel_load(!use_pager);
    Status::SUCCESS
}
