//!
//! scan pci bus
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::qemu;
#[cfg(target_arch = "x86_64")]
use virtfw_libhw::pci;
#[cfg(target_arch = "x86_64")]
use virtfw_libhw::pci_x86::PciHostX86;

#[cfg(target_arch = "x86_64")]
fn scan_one_pci_bus<H: pci::PciHost>(pcihost: &H, bus: u8, indent: usize) {
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
        println!("no pci host bridge");
        return;
    };

    println!("scan pci bus");
    scan_one_pci_bus(&pcihost, 0, 2);
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    #[cfg(target_arch = "x86_64")]
    scan_pci_bus_x86();

    #[cfg(not(target_arch = "x86_64"))]
    println!("only x86 is supported right now");

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
