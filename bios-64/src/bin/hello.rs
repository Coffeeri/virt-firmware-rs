#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

use virtfw_bios_64::println;
use virtfw_bios_64::sercon::sercon_init;
use virtfw_libhw::qemuacpi::QemuAcpi;

#[no_mangle]
fn main() {
    sercon_init();

    println!("Hello world!");

    if let Some(acpi) = QemuAcpi::try_new() {
        acpi.poweroff();
    }
}
