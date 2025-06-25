//!
//! list Hand Off Blocks
//!
#![no_main]
#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::println;

#[cfg(feature = "fdt")]
use virtfw_efi_apps::fdt::print_node;
use virtfw_efi_apps::ovmf_log::MemDebugLog;
use virtfw_efi_apps::{hobs, misc, qemu, rawhobs};

fn list_raw(hl: hobs::HobList) {
    println!("hand off blocks");
    for h in hl {
        println!("  {}", h);
    }
}

fn list_mem(hl: hobs::HobList) {
    let mut ml: Vec<rawhobs::HobMemoryAllocation> = hl
        .filter_map(|h| match h {
            hobs::Hob::MemoryAllocation(m) => Some(m),
            _ => None,
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
        .filter_map(|h| match h {
            hobs::Hob::ResourceDescriptor(r) => Some(r),
            _ => None,
        })
        .collect();
    rl.sort_by_key(|r| r.address);

    println!("resources");
    for r in rl {
        println!("  {:08x} +{:08x} {}", r.address, r.length, r.typename());
    }
}

fn get_devtree_address(hl: hobs::HobList) -> Option<u64> {
    let mut iter = hl.filter_map(|h| match h {
        hobs::Hob::Fdt(data) => Some(data),
        _ => None,
    });
    iter.next()
}

#[cfg(feature = "fdt")]
fn print_devtree(fdt_addr: u64) {
    println!("device tree address: 0x{:x}", fdt_addr);
    let res = unsafe { fdt::Fdt::from_ptr(fdt_addr as *const u8) };
    if let Err(e) = res {
        println!("fdt parse error: {e}");
        return;
    }
    let fdt = res.unwrap();

    println!("model: {}", fdt.root().model());
    if let Some(root) = fdt.find_node("/") {
        for c in root.children() {
            print_node(c, 4);
        }
    }
}

#[cfg(not(feature = "fdt"))]
fn print_devtree(_fdt_addr: u64) {
    println!("device tree found (compiled without parser support)");
}

fn devtree(hl: hobs::HobList) {
    let Some(fdt_addr) = get_devtree_address(hl) else {
        println!("no device tree");
        return;
    };

    print_devtree(fdt_addr);
}

fn get_memorylog_address(hl: hobs::HobList) -> Option<u64> {
    let mut iter = hl.filter_map(|h| match h {
        hobs::Hob::MemLog(data) => Some(data),
        _ => None,
    });
    iter.next()
}

fn print_logbuf(logbuf: &[u8]) {
    for line in logbuf.split(|b| b == &b'\n' || b == &b'\r') {
        if line.is_empty() {
            continue;
        }
        if let Ok(s) = core::str::from_utf8(line) {
            println!("{}", s);
        } else {
            println!("[ garbage log line ]");
        }
    }
}

fn memorylog(hl: hobs::HobList) {
    let Some(log_addr) = get_memorylog_address(hl) else {
        println!("no memory log");
        return;
    };

    let Some(log) = MemDebugLog::new(log_addr) else {
        println!("no valid log found");
        return;
    };

    if let Some(ver) = log.hdr.fw_version.split(|b| b == &b'\0').next() {
        if let Ok(s) = core::str::from_utf8(ver) {
            println!("[ firmware version: {} ]", s);
        }
    }

    if log.hdr.truncated != 0 {
        println!("[ firmware log is truncated ]");
    }

    let h = log.hdr.head_offset as usize;
    let t = log.hdr.tail_offset as usize;
    let s = log.hdr.log_size as usize;
    println!("[ head {}, tail {}, size {} ]", h, t, s);

    if h < t {
        if let Some(buf) = log.log.get(h..t) {
            print_logbuf(buf);
        }
    }

    if h > t {
        if let Some(buf) = log.log.get(h..s) {
            print_logbuf(buf);
        }
        if let Some(buf) = log.log.get(0..t) {
            print_logbuf(buf);
        }
    }
}

fn list_hobs() {
    let Some(hl) = hobs::HobList::new() else {
        println!("failed to get hob list");
        return;
    };

    let Some(cmdline) = misc::cmdline("hoblist.efi") else {
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
            println!("  fdt - check for device tree");
            println!("  log - check for memory log");
        }
        "mem" | "memory" => list_mem(hl),
        "res" | "resources" => list_res(hl),
        "fdt" | "devicetree" => devtree(hl),
        "log" | "memorylog" => memorylog(hl),
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
