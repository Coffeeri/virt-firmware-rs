//! x86 msr support
#![cfg(target_arch = "x86_64")]

use core::arch::asm;

pub fn rdmsr(msr: u32) -> u64 {
    let eax: u32;
    let edx: u32;

    unsafe {
        asm!("rdmsr", in("ecx") msr, out("eax") eax, out("edx") edx, options(att_syntax));
    }
    (eax as u64) | ((edx as u64) << 32)
}
