//! x86 ioport access functions
#![cfg(target_arch = "x86_64")]

use core::arch::asm;

pub fn outb(port: u16, value: u8) {
    unsafe { asm!("outb %al, %dx", in("al") value, in("dx") port, options(att_syntax)) }
}

pub fn inb(port: u16) -> u8 {
    unsafe {
        let ret: u8;
        asm!("inb %dx, %al", in("dx") port, out("al") ret, options(att_syntax));
        ret
    }
}

pub fn outw(port: u16, value: u16) {
    unsafe { asm!("outw %ax, %dx", in("ax") value, in("dx") port, options(att_syntax)) }
}

pub fn inw(port: u16) -> u16 {
    unsafe {
        let ret: u16;
        asm!("inw %dx, %ax", in("dx") port, out("ax") ret, options(att_syntax));
        ret
    }
}

pub fn outl(port: u16, value: u32) {
    unsafe { asm!("outl %eax, %dx", in("eax") value, in("dx") port, options(att_syntax)) }
}

pub fn inl(port: u16) -> u32 {
    unsafe {
        let ret: u32;
        asm!("inl %dx, %eax", in("dx") port, out("eax") ret, options(att_syntax));
        ret
    }
}
