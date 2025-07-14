use core::mem::size_of;
use uguid::Guid;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::guids::{EfiCertPkcs7, RawGuid};
use crate::types::EfiTime;

pub const WIN_CERT_TYPE_EFI_GUID: u16 = 0x0EF1;

// EFI_VARIABLE_AUTHENTICATION_2
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct EfiVarAuth2Header {
    pub time: EfiTime,
    pub length: u32,
    pub revision: u16,
    pub certtype: u16,
    pub guid: RawGuid,
}

pub struct EfiVarAuth2<'a> {
    pub header: EfiVarAuth2Header,
    pub pkcs7: &'a [u8],
    pub data: &'a [u8],
}

impl<'a> EfiVarAuth2<'a> {
    pub fn new(auth: &'a [u8]) -> Option<Self> {
        let (header, _) = EfiVarAuth2Header::read_from_prefix(auth).ok()?;
        if header.revision != 0x0200 || header.certtype != WIN_CERT_TYPE_EFI_GUID {
            return None;
        }

        let guid: Guid = header.guid.into();
        if guid != EfiCertPkcs7 {
            return None;
        };

        let pkcs7_offset = size_of::<EfiVarAuth2Header>();
        let data_offset = 16 + header.length as usize;
        let pkcs7 = auth.get(pkcs7_offset..data_offset)?;
        let data = auth.get(data_offset..)?;

        Some(Self {
            header,
            pkcs7,
            data,
        })
    }
}

pub fn auth_to_esl(auth: &[u8]) -> Option<&[u8]> {
    let authvar = EfiVarAuth2::new(auth)?;
    Some(authvar.data)
}
