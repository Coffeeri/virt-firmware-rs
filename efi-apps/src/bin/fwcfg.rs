#![cfg(target_arch = "x86_64")]
//!
//! list fw_cfg files
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::qemu;
use virtfw_libhw::fwcfg;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    if fwcfg::detect() {
        let list = fwcfg::readdir();
        for e in list {
            println!("fwcfg file:  #{}  {}", e.item, e.name);
        }
    }

    if let Some((item, size)) = fwcfg::findfile("etc/e820") {
        println!("fwcfg: etc/e820 is item {item}, size {size}");
    }

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
