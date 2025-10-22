#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

use log::info;
use virtfw_bios_64::println;
use virtfw_bios_64::sercon::sercon_init;
use virtfw_libhw::pci::{PciDevice, PciHost};
use virtfw_libhw::qemuboards::qemu_probe;

fn for_each_pci_dev<F>(pcihost: &dyn PciHost, bus: u8, mut function: F)
where
    F: FnMut(&PciDevice),
{
    for slot in 0..32 {
        for func in 0..8 {
            if let Some(dev) = pcihost.try_device(bus, slot, func) {
                function(&dev);
                if func == 0 && !pcihost.is_mfd(&dev) {
                    break;
                }
            }
        }
    }
}

fn enumerate_one_pci_bus(pcihost: &dyn PciHost, lastbus: &mut u8, bus: u8) {
    for_each_pci_dev(pcihost, bus, |dev| {
        if !pcihost.is_pci_bridge(dev) {
            return;
        }
        let mut sec = pcihost.bridge_secondary(dev);
        let sub = pcihost.bridge_subordinate(dev);
        if sec == 0 {
            *lastbus += 1;
            sec = *lastbus;
            info!("{}: [assign] secondary bus nr {sec:02x}", dev.addr);
            pcihost.set_bridge_primary(dev, bus);
            pcihost.set_bridge_secondary(dev, sec);
            pcihost.set_bridge_subordinate(dev, 0xff);
            enumerate_one_pci_bus(pcihost, lastbus, sec);
            pcihost.set_bridge_subordinate(dev, *lastbus);
            if sec != *lastbus {
                info!("{}: [assign] subordinate bus nr {:02x}", dev.addr, *lastbus);
            }
        } else {
            info!("{}: [exists] secondary bus nr {sec:02x}", dev.addr);
            if *lastbus < sec {
                *lastbus = sec;
            }
            if sec != sub {
                info!("{}: [exists] subordinate bus nr {sub:02x}", dev.addr);
                if *lastbus < sub {
                    *lastbus = sub;
                }
            }
            enumerate_one_pci_bus(pcihost, lastbus, sec);
        }
    })
}

fn scan_one_pci_bus(pcihost: &dyn PciHost, bus: u8, indent: usize) {
    for_each_pci_dev(pcihost, bus, |dev| {
        println!("{:i$}{}", "", dev, i = indent);

        if pcihost.is_pci_bridge(dev) {
            if let Some(range) = pcihost.bridge_io(dev) {
                println!("{:i$}{}", "", range, i = indent + 2);
            }
            if let Some(range) = pcihost.bridge_mem(dev) {
                println!("{:i$}{}", "", range, i = indent + 2);
            }
            if let Some(range) = pcihost.bridge_prefmem(dev) {
                println!("{:i$}{}", "", range, i = indent + 2);
            }
            let sec = pcihost.bridge_secondary(dev);
            scan_one_pci_bus(pcihost, sec, indent + 4);
        }

        if pcihost.is_device(dev) {
            for b in 0..6 {
                if let Some(range) = pcihost.bar(dev, b) {
                    println!("{:i$}bar {}: {}", "", b, range, i = indent + 2);
                }
            }
        }
    })
}

fn scan_pci_bus(pcihost: &dyn PciHost) {
    info!("enumerate x86 pci bus");
    let mut lastbus = 0;
    enumerate_one_pci_bus(pcihost, &mut lastbus, 0);

    info!("scan x86 pci bus");
    scan_one_pci_bus(pcihost, 0, 2);
}

#[no_mangle]
fn main() {
    sercon_init();

    let Some(board) = qemu_probe() else {
        info!("could not identify qemu hardware");
        return;
    };

    let pcihost = board.pcihost();
    scan_pci_bus(pcihost);

    board.init_acpi();
    board.poweroff();
}
