#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

extern crate alloc;
use log::info;

use virtfw_bios_64::alloc::alloc_init;
use virtfw_bios_64::println;
use virtfw_bios_64::sercon::sercon_init;
use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::{acpi, default_static_alloc, smbios};
use virtfw_libhw::pci::setup::PciSetup;
use virtfw_libhw::pci::{pci_foreach_device, PciHost};
use virtfw_libhw::qemuboards::qemu_probe;

fn scan_one_pci_bus(pcihost: &dyn PciHost, bus: u8, indent: usize) {
    pci_foreach_device(pcihost, bus, |dev| {
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
    info!("enumerate pci bus");
    let mut setup = PciSetup::new(pcihost);
    setup.enumerate_all();

    info!("scanning pci bus [before]");
    scan_one_pci_bus(pcihost, 0, 2);

    info!("setting up pci bus");
    setup.configure_mem32(0xc000_0000..0xe000_0000);
    setup.configure_mem64(0xc_0000_0000..0x10_0000_0000);
    setup.resources_all();
    setup.assign_all();
    setup.apply_all();

    info!("scanning pci bus [after]");
    scan_one_pci_bus(pcihost, 0, 2);
}

#[no_mangle]
fn main() {
    sercon_init(log::LevelFilter::Debug);
    alloc_init();

    let Some(board) = qemu_probe() else {
        info!("could not identify qemu hardware");
        return;
    };
    board.init_acpi();

    let pcihost = board.pcihost();
    scan_pci_bus(pcihost);

    let fwcfg = FwCfgX86::new();
    if let Some(mut smbios) = smbios::SmbiosLoader::new(&fwcfg) {
        smbios
            .load_tables(default_static_alloc)
            .expect("load tables");
        let anchor = smbios.anchor().expect("get anchor");
        info!("smbios: anchor at {anchor:?}");
    }

    if let Some(mut acpi) = acpi::AcpiLoader::new(&fwcfg) {
        acpi.load_tables(default_static_alloc).expect("load tables");
        acpi.run_commands().expect("relocate tables");
        let rsdp = acpi.rsdp().expect("get rsdp");
        info!("acpi: rsdp at {rsdp:?}");
    }

    board.poweroff();
}
