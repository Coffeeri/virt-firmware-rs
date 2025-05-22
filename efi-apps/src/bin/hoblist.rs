//!
//! list Hand Off Blocks
//!
#![no_main]
#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::{hobs, misc, qemu, rawhobs};

fn list_raw(hl: hobs::HobList) {
    println!("hand off blocks");
    for h in hl {
        println!("  {}", h);
    }
}

fn list_mem(hl: hobs::HobList) {
    let mut ml: Vec<rawhobs::HobMemoryAllocation> = hl
        .filter_map(|h| {
            if let hobs::Hob::MemoryAllocation(m) = h {
                Some(m)
            } else {
                None
            }
        })
        .collect();
    ml.sort_by_key(|m| m.address);

    println!("memory allocations");
    for m in ml {
        println!("  {:08x} +{:08x} {}", m.address, m.length, m.typename());
    }
}

fn list_res(hl: hobs::HobList) {
    let mut rl: Vec<rawhobs::HobResourceDescriptor> = hl
        .filter_map(|h| {
            if let hobs::Hob::ResourceDescriptor(m) = h {
                Some(m)
            } else {
                None
            }
        })
        .collect();
    rl.sort_by_key(|r| r.address);

    println!("resources");
    for r in rl {
        println!("  {:08x} +{:08x} {}", r.address, r.length, r.typename());
    }
}

fn list_hobs() {
    let Some(hl) = hobs::HobList::new() else {
        println!("failed to get hob list");
        return;
    };

    let Some(cmdline) = misc::cmdline() else {
        list_raw(hl);
        return;
    };

    match cmdline.as_str() {
        "help" => {
            println!("list firmware hobs");
            println!("modes:");
            println!("  raw - list all hobs (default)");
            println!("  mem - list memory allocations");
            println!("  res - list resource descriptions");
        }
        "mem" | "memory" => list_mem(hl),
        "res" | "resources" => list_res(hl),
        _ => list_raw(hl),
    }
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    list_hobs();

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
