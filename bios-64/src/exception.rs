use crate::println;

#[repr(C)]
pub struct StackFrame {
    // stub in idt.S
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub exception_number: u64,

    // cpu or stub
    pub error_code: u64,

    // cpu
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,

    // cpu (when switching stacks)
    pub rsp: u64,
    pub ss: u64,
}

/// # Safety
///
/// This function is called from exception handler asm stubs which
/// pass a pointer to a `StackFrame` structure.
#[no_mangle]
pub unsafe extern "C" fn exception_handler(regs: *mut StackFrame) {
    let nr = unsafe { (*regs).exception_number };
    let rip = unsafe { (*regs).rip };
    panic!("unhandled exception #{nr} at 0x{rip:x}")
}

extern "C" {
    fn idt_load();
}

pub fn exception_init() {
    unsafe { idt_load() }
    println!("exception handlers installed");
}
