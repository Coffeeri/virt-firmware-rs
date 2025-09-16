//! parse authenticated variable updates
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use core::mem::size_of;
use uguid::Guid;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::efivar::types::EfiVar;
use crate::guids::{EfiCertPkcs7, RawGuid};
use crate::types::EfiTime;

pub const WIN_CERT_TYPE_EFI_GUID: u16 = 0x0EF1;

/// EFI_VARIABLE_AUTHENTICATION_2 in edk2
#[repr(C)]
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
    pub fn new_from_bytes(auth: &'a [u8]) -> Option<Self> {
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

    /// Return pkcs7 signature.  Add envelope if needed.
    pub fn get_pkcs7(&self) -> Vec<u8> {
        let wrap: [u8; 13] = [
            0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x07, 0x02, 0xa0, 0x82,
        ];

        if self.pkcs7.is_empty() {
            return self.pkcs7.to_vec();
        }
        if self.pkcs7.get(4..17) == Some(&wrap) {
            return self.pkcs7.to_vec();
        }

        let data_len_inner = self.pkcs7.len() as u16;
        let data_len_outer = data_len_inner + 15;

        // UEFI spec allows pkcs7 signatures being used without the
        // envelope which identifies them as pkcs7 signatures.  Most
        // crypto libs will not parse them without the envelope though.
        // So add it if needed.
        let mut pkcs7 = vec![0x30, 0x82];
        pkcs7.extend_from_slice(&data_len_outer.to_be_bytes());
        pkcs7.extend_from_slice(&wrap);
        pkcs7.extend_from_slice(&data_len_inner.to_be_bytes());
        pkcs7.extend_from_slice(self.pkcs7);
        pkcs7
    }

    /// Create signed data for pkcs7 signature verification,
    pub fn get_signed_data(&self, variable: &EfiVar) -> Vec<u8> {
        let mut sdata: Vec<u8> = Vec::new();

        // Variable Name (without terminating \0)
        let name_bytes: Vec<u8> = variable
            .name
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        sdata.extend_from_slice(&name_bytes);

        // Variable Namespace Guid
        sdata.extend_from_slice(&variable.guid.to_bytes());

        // Attributes
        let attr: u32 = u32::from(variable.attr);
        sdata.extend_from_slice(&attr.to_le_bytes());

        // TimeStamp
        sdata.extend_from_slice(self.header.time.as_bytes());

        // Variable Content
        sdata.extend_from_slice(self.data);

        sdata
    }
}

/// Unpack efi signature database from authenticated variable update.
pub fn auth_to_esl(auth: &[u8]) -> Option<&[u8]> {
    let authvar = EfiVarAuth2::new_from_bytes(auth)?;
    Some(authvar.data)
}
