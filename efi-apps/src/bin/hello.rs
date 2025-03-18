#!cfg[(target_os = "uefi")]
//!
//! mandatory hello world application
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use efi_apps::qemu;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    println!("Hello world!");

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
