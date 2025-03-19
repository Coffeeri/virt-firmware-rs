#!cfg[(target_os = "uefi")]
//!
//! http network boot config utility
//!
//! usage (efi shell prompt):
//!   `netboot.efi $url`
//!
//! usage (qemu command line):
//!   `qemu -kernel netboot.efi -append $url`
//!
#![no_main]
#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::println;
use uefi::proto::device_path::build;
use uefi::proto::device_path::DevicePath;
use uefi::runtime::VariableAttributes;
use uefi::runtime::VariableVendor;
use uefi::CString16;

use efi_apps::misc;

fn bootentry(title: &str, devpath: &DevicePath, optdata: Option<&[u8]>) -> Vec<u8> {
    let mut bootentry = Vec::new();
    let devpath_bytes = devpath.as_bytes();
    let title_cstr16 = CString16::try_from(title).unwrap();
    bootentry.extend(u32::to_le_bytes(1)); // attribute: active
    bootentry.extend(u16::to_le_bytes(devpath_bytes.len() as u16));
    bootentry.extend(title_cstr16.as_bytes());
    bootentry.extend(devpath_bytes);
    if let Some(o) = optdata {
        bootentry.extend(o);
    }
    bootentry
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let bootnext = CString16::try_from("BootNext").unwrap();
    if runtime::variable_exists(&bootnext, &VariableVendor::GLOBAL_VARIABLE).unwrap() {
        // we land here after reset when loaded via 'qemu -kernel netboot.efi'
        println!("BootNext exists, nothing to do.");
        boot::stall(1_000_000);
        return Status::SUCCESS;
    };

    let Some(uri) = misc::cmdline() else {
        println!("usage: netboot.efi $url");
        boot::stall(1_000_000);
        return Status::SUCCESS;
    };

    println!("Creating BootEntry ...");
    println!("  booturi : {}", uri);

    let title = uri.split('/').last().unwrap();
    println!("  title   : {}", title);

    let mut vec = Vec::new();
    let devpath = build::DevicePathBuilder::with_vec(&mut vec)
        .push(&build::messaging::Uri {
            value: uri.as_bytes(),
        })
        .unwrap()
        .finalize()
        .unwrap();
    println!("  devpath : {:?}", devpath);

    let battr = VariableAttributes::BOOTSERVICE_ACCESS
        | VariableAttributes::RUNTIME_ACCESS
        | VariableAttributes::NON_VOLATILE;
    let boot99 = CString16::try_from("Boot0099").unwrap();
    let bentry = bootentry(title, devpath, None);

    let rc = runtime::set_variable(&boot99, &VariableVendor::GLOBAL_VARIABLE, battr, &bentry);
    println!("Set Boot0099: {:?}", rc);
    if let Err(e) = rc {
        return e.status();
    }

    let rc = runtime::set_variable(
        &bootnext,
        &VariableVendor::GLOBAL_VARIABLE,
        battr,
        &u16::to_le_bytes(0x99),
    );
    println!("Set BootNext: {:?}", rc);
    if let Err(e) = rc {
        return e.status();
    }

    println!("");
    println!("Ready, rebooting in 3 sec.");
    boot::stall(3_000_000);
    runtime::reset(runtime::ResetType::COLD, Status::SUCCESS, None);
}
