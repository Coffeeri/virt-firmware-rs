#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use virtfw_libhw::qemuacpi::QemuAcpi;
use virtfw_libhw::serial::{SerialWriter, SERIAL0_BASE};

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
fn main() {
    let mut s = SerialWriter::new(SERIAL0_BASE);

    let _ = writeln!(s);
    let _ = writeln!(s, "Hello rust world!");
    let _ = writeln!(s);

    if let Some(acpi) = QemuAcpi::try_new() {
        acpi.poweroff();
    }
}
