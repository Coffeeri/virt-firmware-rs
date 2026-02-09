//! simple serial port (16550) driver
#![cfg(target_arch = "x86_64")]

use core::fmt::Write;
use log::{Log, Metadata, Record};

use crate::ioport::{inb, outb};
use crate::mmio::{readb, writeb};

pub const SERIAL0_BASE: u16 = 0x3f8;
pub const SERIAL1_BASE: u16 = 0x2f8;

pub enum SerialWriter {
    IoBase(u16),
    MmioBase(u64),
}

impl SerialWriter {
    pub const fn new_io(iobase: u16) -> SerialWriter {
        SerialWriter::IoBase(iobase)
    }

    pub const fn new_mm(mmiobase: u64) -> SerialWriter {
        SerialWriter::MmioBase(mmiobase)
    }

    fn send(&self, offset: u8, data: u8) {
        match self {
            SerialWriter::IoBase(iobase) => {
                outb(iobase + offset as u16, data);
            }
            SerialWriter::MmioBase(mmiobase) => {
                writeb(mmiobase + offset as u64, data);
            }
        }
    }

    fn recv(&self, offset: u8) -> u8 {
        match self {
            SerialWriter::IoBase(iobase) => inb(iobase + offset as u16),
            SerialWriter::MmioBase(mmiobase) => readb(mmiobase + offset as u64),
        }
    }

    pub fn init(&self) {
        self.send(3, 0x03); // 8N1
        self.send(1, 0); // no irq
    }

    pub fn putc(&self, c: u8) {
        loop {
            let lsr = self.recv(5);
            if lsr & 0x20 != 0 {
                break;
            }
        }
        self.send(0, c);
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
