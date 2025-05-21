//!
//! list Hand Off Blocks
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::{hobs, qemu};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    if let Some(hl) = hobs::HobList::new() {
        for h in hl {
            println!("hob: {}", h);
        }
    }

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
