//!
//! mandatory hello world application
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;
use uefi::system::with_config_table;
use uefi::table::cfg::ConfigTableEntry;

use virtfw_efi_apps::ovmf_log::MemDebugLog;
use virtfw_efi_apps::qemu;
use virtfw_libefi::guids::OvmfMemDebugLog;

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

fn print_debuglog(cfg: ConfigTableEntry) {
    let Some(log) = MemDebugLog::new(cfg.address as u64) else {
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

fn find_debuglog(entries: &[ConfigTableEntry]) -> Option<ConfigTableEntry> {
    let e = entries.iter().find(|e| e.guid == OvmfMemDebugLog)?;
    Some(e.clone())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let Some(entry) = with_config_table(find_debuglog) else {
        println!("no debuglog found");
        return Status::SUCCESS;
    };
    print_debuglog(entry);

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
