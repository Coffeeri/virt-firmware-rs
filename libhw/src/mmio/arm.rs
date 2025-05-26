#![cfg(target_arch = "aarch64")]

use core::arch::asm;

pub fn writeb(addr: u64, value: u8) {
    unsafe { asm!("strb {value:w}, [{addr}]", value = in(reg) value, addr = in(reg) addr) }
}

pub fn readb(addr: u64) -> u8 {
    unsafe {
        let ret: u8;
        asm!("ldrb {value:w}, [{addr}]", addr = in(reg) addr, value = out(reg) ret);
        ret
    }
}

pub fn writew(addr: u64, value: u16) {
    unsafe { asm!("strh {value:w}, [{addr}]", value = in(reg) value, addr = in(reg) addr) }
}

pub fn readw(addr: u64) -> u16 {
    unsafe {
        let ret: u16;
        asm!("ldrh {value:w}, [{addr}]", addr = in(reg) addr, value = out(reg) ret);
        ret
    }
}

pub fn writel(addr: u64, value: u32) {
    unsafe { asm!("str {value:w}, [{addr}]", value = in(reg) value, addr = in(reg) addr) }
}

pub fn readl(addr: u64) -> u32 {
    unsafe {
        let ret: u32;
        asm!("ldr {value:w}, [{addr}]", addr = in(reg) addr, value = out(reg) ret);
        ret
    }
}

pub fn writeq(addr: u64, value: u64) {
    unsafe { asm!("str {value:x}, [{addr}]", value = in(reg) value, addr = in(reg) addr) }
}

pub fn readq(addr: u64) -> u64 {
    unsafe {
        let ret: u64;
        asm!("ldr {value:x}, [{addr}]", addr = in(reg) addr, value = out(reg) ret);
        ret
    }
}
