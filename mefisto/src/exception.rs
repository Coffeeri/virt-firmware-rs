use log::{error, info};
use x86_64::registers::control::{Cr0, Cr2, Cr3, Cr4};
use x86_64::structures::idt::{ExceptionVector, InterruptDescriptorTable, PageFaultErrorCode};
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

fn dumpregs(regs: *mut StackFrame) {
    error!(
        "rax 0x{:016x} rbx 0x{:016x} rcx 0x{:016x}",
        unsafe { (*regs).rax },
        unsafe { (*regs).rbx },
        unsafe { (*regs).rcx },
    );
    error!(
        "rdx 0x{:016x} rsi 0x{:016x} rdi 0x{:016x}",
        unsafe { (*regs).rdx },
        unsafe { (*regs).rsi },
        unsafe { (*regs).rdi },
    );
    error!(
        "rdp 0x{:016x} r8  0x{:016x} r9  0x{:016x}",
        unsafe { (*regs).rbp },
        unsafe { (*regs).r8 },
        unsafe { (*regs).r9 },
    );
    error!(
        "r10 0x{:016x} r11 0x{:016x} r12 0x{:016x}",
        unsafe { (*regs).r10 },
        unsafe { (*regs).r11 },
        unsafe { (*regs).r12 },
    );
    error!(
        "r13 0x{:016x} r14 0x{:016x} r15 0x{:016x}",
        unsafe { (*regs).r13 },
        unsafe { (*regs).r14 },
        unsafe { (*regs).r15 },
    );

    let (cr3, _) = Cr3::read_raw();
    error!(
        "cr0 0x{:016x} cr3 0x{:016x} cr4 0x{:016x}",
        Cr0::read_raw(),
        cr3.start_address(),
        Cr4::read_raw(),
    );
}

fn pagefault(ec: u64, rip: u64) {
    let cr2 = Cr2::read_raw();
    let pfec = PageFaultErrorCode::from_bits(ec).unwrap();
    error!("page fault: rip=0x{rip:x} cr2=0x{cr2:x} code=0x{ec:x}");
    error!("page fault: {pfec:?}");
}

/// # Safety
///
/// This function is called from exception handler asm stubs which
/// pass a pointer to a `StackFrame` structure.
#[no_mangle]
pub unsafe extern "C" fn exception_handler(regs: *mut StackFrame) {
    let nr = unsafe { (*regs).exception_number };
    let ec = unsafe { (*regs).error_code };
    let rip = unsafe { (*regs).rip };
    let exception = ExceptionVector::try_from(nr as u8).unwrap();

    if exception == ExceptionVector::Page {
        pagefault(ec, rip);
    } else {
        error!("exception: {exception:?} (#{nr}) at rip=0x{rip:x}")
    }

    dumpregs(regs);
    panic!("unhandled exception")
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
