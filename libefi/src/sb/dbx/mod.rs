use uguid::Guid;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::guids::{EfiCertPkcs7, RawGuid};
use crate::types::EfiTime;

pub const DBX_X86_64: &[u8] = include_bytes!("x86_64.auth");
pub const DBX_AARCH64: &[u8] = include_bytes!("aarch64.auth");

// EFI_VARIABLE_AUTHENTICATION_2
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
struct EfiVarAuth2 {
    time: EfiTime,
    length: u32,
    revision: u16,
    certtype: u16,
    guid: RawGuid,
}

pub fn auth_to_esl(auth: &[u8]) -> Option<&[u8]> {
    let (header, _) = EfiVarAuth2::read_from_prefix(auth).ok()?;
    let guid: Guid = header.guid.into();
    if guid != EfiCertPkcs7 {
        return None;
    };
    let split_pos = 16 + header.length as usize;
    if split_pos > auth.len() {
        return None;
    }
    let (_, esl) = auth.split_at(split_pos);
    Some(esl)
}
