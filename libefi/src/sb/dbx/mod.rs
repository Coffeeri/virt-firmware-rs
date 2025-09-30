use crate::arch::EfiArch;

pub const DBX_X86_64: &[u8] = include_bytes!("x86_64.auth");
pub const DBX_AARCH64: &[u8] = include_bytes!("aarch64.auth");

#[cfg(target_arch = "aarch64")]
pub const DBX_NATIVE: Option<&[u8]> = Some(DBX_AARCH64);
#[cfg(target_arch = "x86_64")]
pub const DBX_NATIVE: Option<&[u8]> = Some(DBX_X86_64);
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64",)))]
pub const DBX_NATIVE: Option<&[u8]> = None;

pub fn get_dbx(arch: EfiArch) -> &'static [u8] {
    match arch {
        EfiArch::X64 => DBX_X86_64,
        EfiArch::Aa64 => DBX_AARCH64,
    }
}

pub fn get_dbx_native() -> &'static [u8] {
    get_dbx(EfiArch::native())
}
