#!cfg[(target_arch = "x86_64")]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use virtfw_libhw::ioport::{inb, outb};

const SERIAL_BASE: u16 = 0x3f8;

fn serial_init() {
    outb(SERIAL_BASE + 3, 0x03); // 8N1
    outb(SERIAL_BASE + 1, 0); // no irq
}

fn serial_putc(c: u8) {
    loop {
        let lsr = inb(SERIAL_BASE + 5);
        if lsr & 0x20 != 0 {
            break;
        }
    }
    outb(SERIAL_BASE, c);
}

fn puts(s: &str) {
    for c in s.bytes() {
        serial_putc(c);
    }
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    puts("panic\n");
    loop {}
}

#[no_mangle]
fn main() {
    serial_init();

    puts("\n");
    puts("Hello rust world!\n");
    puts("\n");
}
