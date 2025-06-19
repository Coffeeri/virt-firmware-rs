//!
//! read x86 msrs
//!
#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::println;

use virtfw_efi_apps::qemu;

#[cfg(target_arch = "x86_64")]
use virtfw_libhw::x86msr::rdmsr;

#[cfg(target_arch = "x86_64")]
const MSRS: [u32; 4] = [
    0x1B,       // MSR_IA32_APIC_BASE
    0x3A,       // MSR_IA32_FEATURE_CONTROL
    0x1A0,      // MSR_IA32_MISC_ENABLE
    0xC0000080, // MSR_IA32_EFER
];

#[cfg(target_arch = "x86_64")]
fn print_msr(msr: u32) {
    let value = rdmsr(msr);
    println!("    {:8x} : {:8x}", msr, value);
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    println!("msr list");

    #[cfg(target_arch = "x86_64")]
    for m in MSRS {
        print_msr(m);
    }

    qemu::shutdown_if_direct_kernel_load();
    Status::SUCCESS
}
