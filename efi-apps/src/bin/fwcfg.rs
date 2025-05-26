//!
//! list fw_cfg files
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::qemu;
use virtfw_libhw::fwcfg::FwCfg;

#[cfg(target_arch = "x86_64")]
use virtfw_libhw::fwcfg::x86::FwCfgX86;

#[cfg(feature = "fdt")]
use virtfw_efi_apps::fdt;
#[cfg(feature = "fdt")]
use virtfw_libhw::fwcfg::mmio::FwCfgMmio;

fn show_fwcfg<F: FwCfg>(fwcfg: F) {
    let list = fwcfg.readdir();
    for e in list {
        println!("fwcfg file:  #{}  {}", e.item, e.name);
    }

    if let Some((item, size)) = fwcfg.findfile("etc/e820") {
        println!("fwcfg: etc/e820 is item {item}, size {size}");
    }
}

fn fwcfg() {
    #[cfg(target_arch = "x86_64")]
    {
        let fwcfg = FwCfgX86::new();
        if fwcfg.detect() {
            show_fwcfg(fwcfg);
            return;
        }
    }

    #[cfg(feature = "fdt")]
    {
        let res = fdt::find_addr("qemu,fw-cfg-mmio");
        let addr = match res {
            Err(e) => {
                println!("{e}");
                return;
            }
            Ok(a) => a,
        };

        let fwcfg = FwCfgMmio::new(addr);
        println!("mmio fwcfg at {:x}", addr);
        if fwcfg.detect() {
            show_fwcfg(fwcfg);
            return;
        }
    }

    println!("platform not supported");
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    fwcfg();

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
