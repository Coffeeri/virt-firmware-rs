use bitfield_struct::bitfield;
use core::cmp::min;
use core::fmt::{Display, Formatter, Result};

#[repr(C)]
pub struct CpuidEntry {
    pub eax_in: u32,
    pub ecx_in: u32,
    _reserved1: u64,
    _reserved2: u64,
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    _reserved3: u64,
}

impl Display for CpuidEntry {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "in: {:08x} / ", self.eax_in)?;
        if self.ecx_in != 0 {
            write!(f, "{:08x}", self.ecx_in)?;
        } else {
            write!(f, "-       ")?;
        }
        write!(
            f,
            " out: {:08x} {:08x} {:08x} {:08x}",
            self.eax, self.ebx, self.ecx, self.edx
        )
    }
}

#[repr(C, align(4096))]
pub struct CpuidPage {
    pub count: u32,
    _reserved1: u32,
    _reserved2: u64,
    pub entries: [CpuidEntry; 85],
}

#[link_section = ".snp.cpuid"]
pub static CPUID: CpuidPage = unsafe { core::mem::zeroed() };

// like virtfw_libhw::x86cpuid::cpuid
// but using cpuid page
pub fn snp_cpuid(func: u32, subfn: u32) -> (u32, u32, u32, u32) {
    let count = min(CPUID.count as usize, CPUID.entries.len());

    // search with func + subfn first
    for i in 0..count {
        let e = &CPUID.entries[i];
        if e.eax_in == func && e.ecx_in == subfn {
            return (e.eax, e.ebx, e.ecx, e.edx);
        }
    }

    // failing that try func alone
    for i in 0..count {
        let e = &CPUID.entries[i];
        if e.eax_in == func {
            return (e.eax, e.ebx, e.ecx, e.edx);
        }
    }

    // 404
    (0, 0, 0, 0)
}

// CPUID leaf 0x8000_001F

#[bitfield(u32)]
pub struct CpuidSevEax {
    /// AMD Secure Memory Encryption (SME) active
    pub sme: bool,
    /// AMD Secure Encrypted Virtualization (SEV) active
    pub sev: bool,
    /// Indicates if the PFLUSH MSR is supported.
    pub page_flush_msr: bool,
    /// AMD SEV Encrypted State (SEV-ES) active
    pub sev_es: bool,
    /// AMD SEV Secure Nested Paging (SEV-SNP) is active
    pub sev_snp: bool,
    /// Virtual Machine Privilege Levels (VMPL) available
    pub vmpl: bool,
    /// RMPQUERY instruction is supported
    pub rmp_query: bool,
    /// VMPL Supervisor Shadow Stack is supported
    pub vmpl_sss: bool,

    #[bits(24)]
    _reserved: u64,
}

#[bitfield(u32)]
pub struct CpuidSevEbx {
    #[bits(6)]
    pub cbitpos: u8,
    #[bits(6)]
    pub reduced_phys_bits: u8,
    #[bits(20)]
    _reserved: u64,
}
