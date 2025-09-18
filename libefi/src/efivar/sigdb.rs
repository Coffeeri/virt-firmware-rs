//! efi signature databases

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use uguid::Guid;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::guids;

#[cfg(feature = "x509")]
use crate::sb::certs::info::CertInfo;
#[cfg(feature = "x509")]
use crate::types::EfiTime;

#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
struct EfiSigListHeader {
    kind_guid: guids::RawGuid,
    list_size: u32,
    header_size: u32,
    entry_size: u32,
    // entries following here
}

pub struct EfiSignature {
    pub owner: Guid,
    pub data: Vec<u8>,
}

impl EfiSignature {
    pub fn new(guid: &Guid, data: &[u8]) -> Self {
        Self {
            owner: *guid,
            data: data.to_vec(),
        }
    }

    pub fn new_from_bytes(data: &[u8]) -> Option<Self> {
        let (guid, data) = guids::RawGuid::read_from_prefix(data).ok()?;
        Some(Self {
            owner: guid.into(),
            data: data.to_vec(),
        })
    }
}

pub struct EfiSigList {
    pub kind: Guid,
    pub header_data: Vec<u8>,
    pub entry_size: u32,
    pub entries: Vec<EfiSignature>,
}

impl EfiSigList {
    pub fn new_from_bytes(data: &[u8]) -> Option<(Self, &[u8])> {
        let (header, r) = EfiSigListHeader::read_from_prefix(data).ok()?;
        if header.header_size as usize > r.len() {
            return None;
        }
        if header.entry_size as usize <= 16 {
            return None;
        }

        let signatures_size = (header.list_size as usize)
            .checked_sub(header.header_size as usize)?
            .checked_sub(size_of::<EfiSigListHeader>())?;
        if (signatures_size % header.entry_size as usize) != 0 {
            return None;
        }

        let (header_data, r) = r.split_at(header.header_size as usize);
        let (signatures, r) = r.split_at(signatures_size);
        let entries = signatures
            .chunks(header.entry_size as usize)
            .map(|b| EfiSignature::new_from_bytes(b).unwrap())
            .collect();

        let siglist = Self {
            kind: header.kind_guid.into(),
            header_data: header_data.to_vec(),
            entry_size: header.entry_size,
            entries,
        };

        Some((siglist, r))
    }

    pub fn new_x509_from_der(owner: &Guid, data: &[u8]) -> Self {
        let signature = EfiSignature::new(owner, data);
        Self {
            kind: guids::EfiCertX509,
            header_data: Vec::new(),
            entry_size: (data.len() + 16) as u32,
            entries: vec![signature],
        }
    }

    pub fn new_sha256_from_list(owner: &Guid, list: Vec<[u8; 32]>) -> Self {
        let entries = list
            .into_iter()
            .map(|i| EfiSignature::new(owner, &i))
            .collect();
        Self {
            kind: guids::EfiCertSha256,
            header_data: Vec::new(),
            entry_size: 32 + 16,
            entries,
        }
    }

    pub fn new_pk_external_mgmt() -> Self {
        let data: &[u8] = &[0];
        let signature = EfiSignature::new(&guids::OvmfEnrollDefaultKeys, data);
        Self {
            kind: guids::EfiCertExternalMgmt,
            header_data: Vec::new(),
            entry_size: (data.len() + 16) as u32,
            entries: vec![signature],
        }
    }

    pub fn get_x509(&self) -> Option<&[u8]> {
        if self.kind != guids::EfiCertX509 {
            return None;
        }
        Some(self.entries[0].data.as_ref())
    }
}

impl fmt::Display for EfiSigList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            guids::EfiCertX509 => write!(f, "X.509 certificate"),
            guids::EfiCertSha256 => write!(f, "SHA256 hash list ({} entries)", self.entries.len()),
            _ => write!(f, "Other ({})", guids::pretty_name(&self.kind)),
        }
    }
}

impl From<&EfiSigList> for Vec<u8> {
    fn from(siglist: &EfiSigList) -> Vec<u8> {
        let size = size_of::<EfiSigListHeader>() as u32
            + siglist.header_data.len() as u32
            + siglist.entries.len() as u32 * siglist.entry_size;
        let header = EfiSigListHeader {
            kind_guid: siglist.kind.into(),
            list_size: size,
            header_size: siglist.header_data.len() as u32,
            entry_size: siglist.entry_size,
        };

        let mut bytes = Vec::new();
        bytes.extend_from_slice(header.as_bytes());
        bytes.extend_from_slice(&siglist.header_data);
        for sig in &siglist.entries {
            bytes.extend_from_slice(&sig.owner.to_bytes());
            bytes.extend_from_slice(&sig.data);
        }
        bytes
    }
}

pub struct EfiSigDB {
    pub siglists: Vec<EfiSigList>,
}

impl EfiSigDB {
    pub fn new() -> Self {
        let siglists = Vec::new();
        Self { siglists }
    }

    pub fn new_from_bytes(data: &[u8]) -> Option<Self> {
        let mut siglists = Vec::new();

        let mut d = data;
        loop {
            let (siglist, r) = EfiSigList::new_from_bytes(d)?;
            siglists.push(siglist);

            if r.is_empty() {
                break;
            }
            d = r;
        }
        Some(Self { siglists })
    }

    pub fn new_pk_external_mgmt() -> Self {
        let siglist = EfiSigList::new_pk_external_mgmt();
        Self {
            siglists: vec![siglist],
        }
    }

    pub fn add_x509_from_der(&mut self, owner: &Guid, data: &[u8]) {
        let siglist = EfiSigList::new_x509_from_der(owner, data);
        self.siglists.push(siglist);
    }

    pub fn add_sha256_from_list(&mut self, owner: &Guid, list: Vec<[u8; 32]>) {
        let siglist = EfiSigList::new_sha256_from_list(owner, list);
        self.siglists.push(siglist);
    }

    pub fn get_x509_list(&self) -> Vec<&[u8]> {
        self.siglists.iter().filter_map(|s| s.get_x509()).collect()
    }

    #[cfg(feature = "x509")]
    pub fn get_x509_mtime(&self) -> Option<EfiTime> {
        self.siglists
            .iter()
            .filter_map(|s| s.get_x509())
            .filter_map(CertInfo::new_from_der)
            .map(|i| i.not_before)
            .max()
    }
}

impl Default for EfiSigDB {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&EfiSigDB> for Vec<u8> {
    fn from(sigdb: &EfiSigDB) -> Vec<u8> {
        let mut bytes = Vec::new();
        for siglist in &sigdb.siglists {
            let b: Vec<u8> = siglist.into();
            bytes.extend_from_slice(&b);
        }
        bytes
    }
}
