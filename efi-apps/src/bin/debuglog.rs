//!
//! mandatory hello world application
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;
use uefi::system::with_config_table;
use uefi::table::cfg::ConfigTableEntry;

use virtfw_efi_apps::misc;
use virtfw_efi_apps::ovmf_log::MemDebugLog;
use virtfw_efi_apps::pager::Pager;
use virtfw_efi_apps::qemu;
use virtfw_libefi::guids::OvmfMemDebugLog;

fn print_logbuf(logbuf: &[u8], pager: &mut Pager) {
    for line in logbuf.split(|b| b == &b'\n' || b == &b'\r') {
        if line.is_empty() {
            continue;
        }
        if let Ok(s) = core::str::from_utf8(line) {
            println!("{}", s);
        } else {
            println!("[ garbage log line ]");
        }
        pager.lines(1);
    }
}

fn print_debuglog(cfg: ConfigTableEntry, pager: &mut Pager) {
    let Some(log) = MemDebugLog::new(cfg.address as u64) else {
        println!("no valid log found");
        return;
    };

    println!("[ firmware log gpa: 0x{:x} ]", cfg.address as u64);
    pager.lines(1);

    if let Some(ver) = log.hdr.fw_version.split(|b| b == &b'\0').next() {
        if let Ok(s) = core::str::from_utf8(ver) {
            println!("[ firmware version: {} ]", s);
            pager.lines(1);
        }
    }

    if log.hdr.truncated != 0 {
        println!("[ firmware log is truncated ]");
        pager.lines(1);
    }

    let h = log.hdr.head_offset as usize;
    let t = log.hdr.tail_offset as usize;
    let s = log.hdr.log_size as usize;
    println!("[ head {}, tail {}, size {} ]", h, t, s);
    pager.lines(1);

    if h < t {
        if let Some(buf) = log.log.get(h..t) {
            print_logbuf(buf, pager);
        }
    }

    if h > t {
        if let Some(buf) = log.log.get(h..s) {
            print_logbuf(buf, pager);
        }
        if let Some(buf) = log.log.get(0..t) {
            print_logbuf(buf, pager);
        }
    }
}

fn find_debuglog(entries: &[ConfigTableEntry]) -> Option<ConfigTableEntry> {
    let e = entries.iter().find(|e| e.guid == OvmfMemDebugLog)?;
    Some(e.clone())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let mut use_pager = false;
    if let Some(cmdline) = misc::cmdline("hoblist") {
        if cmdline == "-p" || cmdline == "pager" {
            use_pager = true;
        }
    }

    let mut pager = Pager::new(use_pager);

    if let Some(entry) = with_config_table(find_debuglog) {
        print_debuglog(entry, &mut pager);
    } else {
        println!("no debuglog found");
        pager.lines(1);
    }

    pager.end();
    qemu::shutdown_if_direct_kernel_load(!use_pager);
    Status::SUCCESS
}
