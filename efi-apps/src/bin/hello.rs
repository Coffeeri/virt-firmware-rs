//!
//! mandatory hello world application
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::{misc, qemu};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    println!("Hello world!");
    let _ = misc::cmdline("hello.efi");

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
