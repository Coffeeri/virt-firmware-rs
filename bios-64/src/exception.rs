use log::info;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::VirtAddr;

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

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

extern "C" {
    // see idt.S
    fn stub_de();
    fn stub_br();
    fn stub_ud();
    fn stub_nm();
    fn stub_df();
    fn stub_ts();
    fn stub_np();
    fn stub_ss();
    fn stub_gp();
    fn stub_pf();
    fn stub_mf();
    fn stub_ac();
    fn stub_mc();
    fn stub_xm();
    fn stub_ve();
    fn stub_cp();
    fn stub_hv();
    fn stub_vc();
    fn stub_sx();
}

pub fn exception_init() {
    #[allow(static_mut_refs)]
    unsafe {
        IDT.divide_error
            .set_handler_addr(VirtAddr::new(stub_de as usize as u64));
        IDT.bound_range_exceeded
            .set_handler_addr(VirtAddr::new(stub_br as usize as u64));
        IDT.invalid_opcode
            .set_handler_addr(VirtAddr::new(stub_ud as usize as u64));
        IDT.device_not_available
            .set_handler_addr(VirtAddr::new(stub_nm as usize as u64));
        IDT.double_fault
            .set_handler_addr(VirtAddr::new(stub_df as usize as u64));
        IDT.invalid_tss
            .set_handler_addr(VirtAddr::new(stub_ts as usize as u64));
        IDT.segment_not_present
            .set_handler_addr(VirtAddr::new(stub_np as usize as u64));
        IDT.stack_segment_fault
            .set_handler_addr(VirtAddr::new(stub_ss as usize as u64));
        IDT.general_protection_fault
            .set_handler_addr(VirtAddr::new(stub_gp as usize as u64));
        IDT.page_fault
            .set_handler_addr(VirtAddr::new(stub_pf as usize as u64));
        IDT.x87_floating_point
            .set_handler_addr(VirtAddr::new(stub_mf as usize as u64));
        IDT.alignment_check
            .set_handler_addr(VirtAddr::new(stub_ac as usize as u64));
        IDT.machine_check
            .set_handler_addr(VirtAddr::new(stub_mc as usize as u64));
        IDT.simd_floating_point
            .set_handler_addr(VirtAddr::new(stub_xm as usize as u64));
        IDT.virtualization
            .set_handler_addr(VirtAddr::new(stub_ve as usize as u64));
        IDT.cp_protection_exception
            .set_handler_addr(VirtAddr::new(stub_cp as usize as u64));
        IDT.hv_injection_exception
            .set_handler_addr(VirtAddr::new(stub_hv as usize as u64));
        IDT.vmm_communication_exception
            .set_handler_addr(VirtAddr::new(stub_vc as usize as u64));
        IDT.security_exception
            .set_handler_addr(VirtAddr::new(stub_sx as usize as u64));
        IDT.load();
    }
    info!("exception handlers installed");
}
