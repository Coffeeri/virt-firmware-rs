#![cfg(target_arch = "x86_64")]

use core::arch::asm;

pub fn writeb(addr: u64, value: u8) {
    unsafe { asm!("movb %al, (%rdx)", in("al") value, in("rdx") addr, options(att_syntax)) }
}

pub fn readb(addr: u64) -> u8 {
    unsafe {
        let ret: u8;
        asm!("movb (%rdx), %al", in("rdx") addr, out("al") ret, options(att_syntax));
        ret
    }
}

pub fn writew(addr: u64, value: u16) {
    unsafe { asm!("movw %ax, (%rdx)", in("ax") value, in("rdx") addr, options(att_syntax)) }
}

pub fn readw(addr: u64) -> u16 {
    unsafe {
        let ret: u16;
        asm!("movw (%rdx), %ax", in("rdx") addr, out("ax") ret, options(att_syntax));
        ret
    }
}

pub fn writel(addr: u64, value: u32) {
    unsafe { asm!("movl %eax, (%rdx)", in("eax") value, in("rdx") addr, options(att_syntax)) }
}

pub fn readl(addr: u64) -> u32 {
    unsafe {
        let ret: u32;
        asm!("movl (%rdx), %eax", in("rdx") addr, out("eax") ret, options(att_syntax));
        ret
    }
}

pub fn writeq(addr: u64, value: u64) {
    unsafe { asm!("movq %rax, (%rdx)", in("rax") value, in("rdx") addr, options(att_syntax)) }
}

pub fn readq(addr: u64) -> u64 {
    unsafe {
        let ret: u64;
        asm!("movq (%rdx), %rax", in("rdx") addr, out("rax") ret, options(att_syntax));
        ret
    }
}
