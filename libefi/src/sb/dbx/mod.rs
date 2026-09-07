use crate::arch::EfiArch;

pub const DBX_2011_X86_64: &[u8] = include_bytes!("2011.x86_64.auth");
pub const DBX_2011_AARCH64: &[u8] = include_bytes!("2011.aarch64.auth");

pub fn get_dbx(arch: EfiArch) -> &'static [u8] {
    match arch {
        EfiArch::X64 => DBX_2011_X86_64,
        EfiArch::Aa64 => DBX_2011_AARCH64,
    }
}

pub fn get_dbx_native() -> &'static [u8] {
    get_dbx(EfiArch::native())
}

pub fn get_dbx_opt(arch: Option<EfiArch>) -> &'static [u8] {
    match arch {
        Some(a) => get_dbx(a),
        None => get_dbx_native(),
    }
}
