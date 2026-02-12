use bitfield_struct::bitfield;
use core::arch::asm;
use core::mem::offset_of;

use virtfw_libhw::x86msr::{rdmsr, wrmsr};

pub const MSR_SEV_GHCB: u32 = 0xc001_0130;

pub fn vmgexit() {
    unsafe {
        asm!("rep vmmcall");
    }
}

#[bitfield(u64)]
pub struct MsrGhcbTerminate {
    #[bits(12, default = 0x100)]
    pub request: u16,
    #[bits(4)]
    pub reason_set: u8,
    #[bits(8)]
    pub reason_code: u8,
    #[bits(40)]
    pub _reserved: u64,
}

#[bitfield(u64)]
pub struct MsrGhcbSevInfoRsp {
    #[bits(12, default = 0x001)]
    pub request: u16,
    #[bits(12)]
    pub _reserved: u64,
    #[bits(8)]
    pub cbitpos: u8,
    #[bits(16)]
    pub min_version: u16,
    #[bits(16)]
    pub max_version: u16,
}

#[bitfield(u64)]
pub struct MsrGhcbSevInfoReq {
    #[bits(12, default = 0x002)]
    pub request: u16,
    #[bits(52)]
    pub _reserved: u64,
}

#[bitfield(u64)]
pub struct MsrGhcbRegisterGpaReq {
    #[bits(12, default = 0x012)]
    pub request: u16,
    #[bits(52)]
    pub gfn: u64,
}

#[bitfield(u64)]
pub struct MsrGhcbRegisterGpaRsp {
    #[bits(12, default = 0x013)]
    pub request: u16,
    #[bits(52)]
    pub gfn: u64,
}

#[bitfield(u64)]
pub struct MsrGhcbPageStateReq {
    #[bits(12, default = 0x014)]
    pub request: u16,
    #[bits(40)]
    pub gfn: u64,
    #[bits(4)]
    pub operation: u8,
    #[bits(8)]
    pub _reserved: u8,
}

#[bitfield(u64)]
pub struct MsrGhcbPageStateRsp {
    #[bits(12, default = 0x015)]
    pub request: u16,
    #[bits(20)]
    pub _reserved: u32,
    #[bits(32)]
    pub error_code: u32,
}

#[bitfield(u64)]
pub struct MsrGhcbVmmFeatureReq {
    #[bits(12, default = 0x080)]
    pub request: u16,
    #[bits(52)]
    pub _reserved: u64,
}

#[bitfield(u64)]
pub struct MsrGhcbVmmFeatureRsp {
    #[bits(12, default = 0x081)]
    pub request: u16,
    pub snp: bool,
    pub ap_create: bool,
    pub res_inj: bool,
    pub res_inj_timer: bool,
    pub apic_id: bool,
    pub vmpl: bool,
    pub page_state: bool,
    pub tdisp: bool,
    pub ghcb_unregister: bool,
    #[bits(43)]
    pub _reserved: u64,
}

pub fn msr_ghcb_terminate(set: u8, code: u8) -> ! {
    let req = MsrGhcbTerminate::new()
        .with_reason_set(set)
        .with_reason_code(code);
    wrmsr(MSR_SEV_GHCB, req.into());
    vmgexit();
    #[allow(clippy::empty_loop)]
    loop {}
}

pub fn msr_ghcb_sev_info() -> MsrGhcbSevInfoRsp {
    let req = MsrGhcbSevInfoReq::new();
    wrmsr(MSR_SEV_GHCB, req.into());
    vmgexit();
    rdmsr(MSR_SEV_GHCB).into()
}

pub fn msr_ghcb_page_state(gpa: u64, share: bool) {
    let req = MsrGhcbPageStateReq::new()
        .with_gfn(gpa >> 12)
        .with_operation(if share { 2 } else { 1 });
    wrmsr(MSR_SEV_GHCB, req.into());
    vmgexit();
    let rsp: MsrGhcbPageStateRsp = rdmsr(MSR_SEV_GHCB).into();
    if rsp.request() != 0x15 || rsp.error_code() != 0 {
        msr_ghcb_terminate(4, 0x14);
    }
}

pub fn msr_ghcb_vmm_features() -> MsrGhcbVmmFeatureRsp {
    let req = MsrGhcbVmmFeatureReq::new();
    wrmsr(MSR_SEV_GHCB, req.into());
    vmgexit();
    rdmsr(MSR_SEV_GHCB).into()
}

#[repr(C, align(4096))]
pub struct GHCB {
    _r1: [u8; 0xcb],
    pub cpl: u8,
    _r2: [u8; 0x74],
    pub xss: u64,
    _r3: [u8; 0x18],
    pub dr7: u64,
    _r4: [u8; 0x90],
    pub rax: u64,
    _r5: [u8; 0x100],
    _rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    _r6: [u8; 0x70],
    pub exit_code: u64,
    pub exit_info1: u64,
    pub exit_info2: u64,
    pub scratch: u64,
    _r7: [u8; 0x38],
    pub xcr0: u64,
    pub valid_bitmap: [u8; 0x10],
    pub x87_state_gpa: u64,
    _r8: [u8; 0x3f8],
    pub shared_buffer: [u8; 0x7f0],
    _r9: [u8; 0x0a],
    pub ghcb_version: u16,
    pub ghcb_usage: u32,
}

impl GHCB {
    pub fn gpa(&self) -> u64 {
        self as *const Self as u64
    }

    pub fn gfn(&self) -> u64 {
        self.gpa() >> 12
    }

    pub fn register(&self) {
        let req = MsrGhcbRegisterGpaReq::new().with_gfn(self.gfn());
        wrmsr(MSR_SEV_GHCB, req.into());
        vmgexit();
        let rsp: MsrGhcbRegisterGpaRsp = rdmsr(MSR_SEV_GHCB).into();
        if rsp.request() != 0x13 || rsp.gfn() != req.gfn() {
            msr_ghcb_terminate(4, 0x12);
        }
    }

    pub fn reset(&mut self) {
        self.valid_bitmap = [0; 0x10];
        self.ghcb_version = 2;
        self.ghcb_usage = 0;
    }

    pub fn vmmcall(&self) {
        wrmsr(MSR_SEV_GHCB, self.gpa());
        vmgexit();
    }

    fn set_bitmap(&mut self, offset: usize) {
        let qword = offset / 8;
        let index = qword / 8;
        let bmask = 1 << (qword % 8);
        self.valid_bitmap[index] |= bmask;
    }

    pub fn set_rax(&mut self, value: u64) {
        self.set_bitmap(offset_of!(Self, rax));
        self.rax = value;
    }

    pub fn set_exit(&mut self, code: u64, info1: u64, info2: u64) {
        self.set_bitmap(offset_of!(Self, exit_code));
        self.set_bitmap(offset_of!(Self, exit_info1));
        self.set_bitmap(offset_of!(Self, exit_info2));
        self.exit_code = code;
        self.exit_info1 = info1;
        self.exit_info2 = info2;
    }

    pub fn set_scratch(&mut self, value: u64) {
        self.set_bitmap(offset_of!(Self, scratch));
        self.scratch = self.gpa() + offset_of!(Self, shared_buffer) as u64;
        self.shared_buffer[..8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn get_scratch(&self) -> u64 {
        u64::from_le_bytes(self.shared_buffer[..8].try_into().unwrap())
    }
}
