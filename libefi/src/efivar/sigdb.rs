//! efi signature databases

extern crate alloc;
use alloc::vec::Vec;
use core::fmt;

use uguid::Guid;
use zerocopy::{FromBytes, KnownLayout};

use crate::guids;

#[repr(C, packed)]
#[derive(Debug, FromBytes, KnownLayout)]
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

pub struct EfiSigDB {
    pub siglists: Vec<EfiSigList>,
}

impl EfiSigDB {
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
}
