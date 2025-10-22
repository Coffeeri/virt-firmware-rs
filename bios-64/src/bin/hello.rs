#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

use virtfw_bios_64::println;
use virtfw_bios_64::sercon::sercon_init;
use virtfw_libhw::qemuboards::qemu_try_poweroff;

#[no_mangle]
fn main() {
    sercon_init();

    println!("");
    println!("Hello world!");
    println!("");

    qemu_try_poweroff();
}
