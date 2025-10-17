//! simple serial port (16550) driver
#![cfg(target_arch = "x86_64")]

use core::fmt::Write;
use log::{Log, Metadata, Record};

use crate::ioport::{inb, outb};

pub const SERIAL0_BASE: u16 = 0x3f8;
pub const SERIAL1_BASE: u16 = 0x2f8;

pub struct SerialWriter {
    pub iobase: u16,
}

impl SerialWriter {
    pub const fn new(iobase: u16) -> SerialWriter {
        SerialWriter { iobase }
    }

    pub fn init(&self) {
        outb(self.iobase + 3, 0x03); // 8N1
        outb(self.iobase + 1, 0); // no irq
    }

    pub fn putc(&self, c: u8) {
        loop {
            let lsr = inb(self.iobase + 5);
            if lsr & 0x20 != 0 {
                break;
            }
        }
        outb(self.iobase, c);
    }

    pub fn puts<B: AsRef<[u8]> + ?Sized>(&self, buf: &B) {
        for &c in buf.as_ref() {
            self.putc(c);
        }
    }

    pub fn putsln<B: AsRef<[u8]> + ?Sized>(&self, buf: &B) {
        self.puts(buf);
        self.putc(b'\n');
    }
}

impl Write for &SerialWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.puts(&s);
        Ok(())
    }
}

impl Log for SerialWriter {
    fn enabled(&self, _meta: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut w = self;
        let _ = writeln!(w, "{}: {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
