#!cfg[(target_arch = "x86_64")]
//!
//! scan pci bus
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::qemu;
use virtfw_libhw::pci;

fn scan_one_pci_bus(bus: u8, indent: usize) {
    for slot in 0..32 {
        for func in 0..8 {
            if let Some(dev) = pci::PciDevice::try_new(bus, slot, func) {
                println!("{:i$}{}", "", dev, i = indent);

                if dev.is_pci_bridge() {
                    println!("{:i$}brctl: {}", "", dev.bridge_ctl(), i = indent + 2);
                    if let Some(range) = dev.bridge_io() {
                        println!("{:i$}{}", "", range, i = indent + 2);
                    }
                    if let Some(range) = dev.bridge_mem() {
                        println!("{:i$}{}", "", range, i = indent + 2)
                    }
                    if let Some(range) = dev.bridge_prefmem() {
                        println!("{:i$}{}", "", range, i = indent + 2)
                    }
                    let sec = dev.bridge_secondary();
                    scan_one_pci_bus(sec, indent + 4);
                }

                if dev.is_device() {
                    if dev.is_vga() {
                        println!("{:i$}cmd: {}", "", dev.command(), i = indent + 2);
                    }
                    for b in 0..6 {
                        if let Some(range) = dev.bar(b) {
                            println!("{:i$}bar {}: {}", "", b, range, i = indent + 2)
                        }
                    }
                }

                if func == 0 && !dev.is_mfd() {
                    break;
                }
            } else if func == 0 {
                break;
            }
        }
    }
}

fn scan_pci_bus() {
    println!("");
    println!("scan pci bus");
    scan_one_pci_bus(0, 2);
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    if pci::PciDevice::try_new(0, 0, 0).is_some() {
        scan_pci_bus();
    } else {
        println!("no pci host bridge");
    };

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
