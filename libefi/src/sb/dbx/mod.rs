use crate::arch::EfiArch;
use crate::efivar::auth::auth_to_esl;
use crate::efivar::sigdb::EfiSigDB;

pub enum KEK {
    Microsoft2011,
    Microsoft2023,
}

pub const DBX_2011_X86_64: &[u8] = include_bytes!("2011.x86_64.auth");
pub const DBX_2011_AARCH64: &[u8] = include_bytes!("2011.aarch64.auth");
pub const DBX_2023_X86_64: &[u8] = include_bytes!("2023.x86_64.auth");
pub const DBX_2023_AARCH64: &[u8] = include_bytes!("2023.aarch64.auth");

/// get (signed) dbx update blob
pub fn get_dbx_update(arch: EfiArch, kek: KEK) -> &'static [u8] {
    match (arch, kek) {
        (EfiArch::X64, KEK::Microsoft2011) => DBX_2011_X86_64,
        (EfiArch::Aa64, KEK::Microsoft2011) => DBX_2011_AARCH64,
        (EfiArch::X64, KEK::Microsoft2023) => DBX_2023_X86_64,
        (EfiArch::Aa64, KEK::Microsoft2023) => DBX_2023_AARCH64,
    }
}

/// get merged dbx signature list
pub fn get_dbx_merged(arch: EfiArch) -> EfiSigDB {
    let (_, dbx11) = auth_to_esl(get_dbx_update(arch, KEK::Microsoft2011)).unwrap();
    let (_, dbx23) = auth_to_esl(get_dbx_update(arch, KEK::Microsoft2023)).unwrap();
    let add11 = EfiSigDB::new_from_bytes(dbx11).unwrap();
    let add23 = EfiSigDB::new_from_bytes(dbx23).unwrap();

    let mut sigdb = EfiSigDB::new();
    sigdb.merge(&add11);
    sigdb.merge(&add23);
    sigdb
}

pub fn get_dbx_native() -> EfiSigDB {
    get_dbx_merged(EfiArch::native())
}

pub fn get_dbx_opt(arch: Option<EfiArch>) -> EfiSigDB {
    match arch {
        Some(a) => get_dbx_merged(a),
        None => get_dbx_native(),
    }
}
