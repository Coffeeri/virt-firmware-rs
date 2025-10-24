//! x86 cpuid support
#![cfg(target_arch = "x86_64")]

use core::arch::asm;

/// run cpuid instruction and return (eax, ebx, ecx, edx) registers
pub fn cpuid(func: u32, subfn: u32) -> (u32, u32, u32, u32) {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;

    unsafe {
        // LLVM uses rbx, so save and restore rbx.
        // Also use edi to return the value.
        asm!("push %rbx",
             "cpuid",
             "movl %ebx, %edi",
             "pop %rbx",
             in("eax") func,
             in("ecx") subfn,
             lateout("eax") eax,
             out("edi") ebx,
             lateout("ecx") ecx,
             out("edx") edx,
             options(att_syntax));
    }
    (eax, ebx, ecx, edx)
}

/// returns true if the CPU supports gigabyte pages.
pub fn has_gb_pages() -> bool {
    let (max, _, _, _) = cpuid(0x80000000, 0);
    if max < 0x80000001 {
        return false;
    }
    let (_, _, _, edx) = cpuid(0x80000001, 0);
    edx & (1 << 26) != 0
}

pub fn has_rdrand() -> bool {
    let (_, _, ecx, _) = cpuid(0x1, 0);
    ecx & (1 << 30) != 0
}

// not really a good fit, but hey, you need the cpuid check anyway ...
pub fn rdrand() -> u64 {
    let rax: u64;
    unsafe {
        asm!("rdrand %rax",
             out("rax") rax,
             options(att_syntax));
    }
    rax
}
