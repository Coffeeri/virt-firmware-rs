#![cfg(target_arch = "x86_64")]

use core::fmt::Write;

use crate::ioport::{inb, outb};

pub const SERIAL0_BASE: u16 = 0x3f8;
pub const SERIAL1_BASE: u16 = 0x2f8;

pub struct SerialWriter {
    iobase: u16,
}

impl SerialWriter {
    pub fn new(iobase: u16) -> SerialWriter {
        let s = SerialWriter { iobase };
        s.init();
        s
    }

    fn init(&self) {
        outb(self.iobase + 3, 0x03); // 8N1
        outb(self.iobase + 1, 0); // no irq
    }

    fn putc(&self, c: u8) {
        loop {
            let lsr = inb(self.iobase + 5);
            if lsr & 0x20 != 0 {
                break;
            }
        }
        outb(self.iobase, c);
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for c in s.bytes() {
            self.putc(c);
        }
        Ok(())
    }
}
