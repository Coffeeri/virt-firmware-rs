#!cfg[(target_arch = "x86_64") && (target_os = "uefi")]
//!
//! list fw_cfg files
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use efi_apps::qemu;
use libhw::fwcfg;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    if fwcfg::detect() {
        let list = fwcfg::readdir();
        for e in list {
            println!("fwcfg file: {}", e.name);
        }
    }

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
