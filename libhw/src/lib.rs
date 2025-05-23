#![cfg(target_arch = "x86_64")]
#![no_std]

pub mod fwcfg;
pub mod ioport;
pub mod mmio_x86;
pub mod pci;
pub mod qemuacpi;
pub mod serial;
