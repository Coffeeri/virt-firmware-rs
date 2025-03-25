//!
//! list fw_cfg files
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::qemu;
#[cfg(target_arch = "x86_64")]
use virtfw_libhw::fwcfg;

#[cfg(target_arch = "x86_64")]
fn show_fwcfg() {
    if fwcfg::detect() {
        let list = fwcfg::readdir();
        for e in list {
            println!("fwcfg file:  #{}  {}", e.item, e.name);
        }
    }

    if let Some((item, size)) = fwcfg::findfile("etc/e820") {
        println!("fwcfg: etc/e820 is item {item}, size {size}");
    }
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    #[cfg(target_arch = "x86_64")]
    show_fwcfg();

    #[cfg(not(target_arch = "x86_64"))]
    println!("only x86 is supported right now");

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
