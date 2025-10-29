extern crate alloc;
use alloc::vec::Vec;
use core::mem::size_of;
use log::debug;
use zerocopy::FromBytes;

use crate::fwcfg::FwCfg;

/*
  UINT8     AnchorString[SMBIOS_ANCHOR_STRING_LENGTH];
  UINT8     EntryPointStructureChecksum;
  UINT8     EntryPointLength;
  UINT8     MajorVersion;
  UINT8     MinorVersion;
  UINT16    MaxStructureSize;
  UINT8     EntryPointRevision;
  UINT8     FormattedArea[5];
  UINT8     IntermediateAnchorString[5];
  UINT8     IntermediateChecksum;
  UINT16    TableLength;
  UINT32    TableAddress;
  UINT16    NumberOfSmbiosStructures;
  UINT8     SmbiosBcdRevision;
*/

#[derive(Debug, FromBytes)]
#[repr(C, packed)]
struct SmbiosV2 {
    pub anchor: [u8; 4],
    pub checksum: u8,
    pub length: u8,
    pub major: u8,
    pub minor: u8,
    pub maxsize: u16,
    pub revision: u8,
    pub fa: [u8; 5],
    pub ias: [u8; 5],
    pub ic: u8,
    pub tablelength: u16,
    pub tableaddress: u32,
    pub count: u16,
    pub bcdrevision: u8,
}

/*
  UINT8     AnchorString[SMBIOS_3_0_ANCHOR_STRING_LENGTH];
  UINT8     EntryPointStructureChecksum;
  UINT8     EntryPointLength;
  UINT8     MajorVersion;
  UINT8     MinorVersion;
  UINT8     DocRev;
  UINT8     EntryPointRevision;
  UINT8     Reserved;
  UINT32    TableMaximumSize;
  UINT64    TableAddress;
*/

#[derive(Debug, FromBytes)]
#[repr(C, packed)]
struct SmbiosV3 {
    pub anchor: [u8; 5],
    pub checksum: u8,
    pub length: u8,
    pub major: u8,
    pub minor: u8,
    pub docrev: u8,
    pub revision: u8,
    pub _reserved: u8,
    pub maxsize: u32,
    pub address: u64,
}

pub struct SmbiosLoader<'f> {
    fwcfg: &'f dyn FwCfg,
    major: u8,
    minor: u8,
}

impl<'f> SmbiosLoader<'f> {
    pub fn new(fwcfg: &'f dyn FwCfg) -> Option<Self> {
        let anchor = fwcfg.load_file("etc/smbios/smbios-anchor")?;
        let major;
        let minor;

        if anchor.len() == size_of::<SmbiosV2>() {
            let v2 = SmbiosV2::read_from_bytes(&anchor).unwrap();
            debug!("{v2:?}");
            major = v2.major;
            minor = v2.minor;
        } else if anchor.len() == size_of::<SmbiosV3>() {
            let v3 = SmbiosV3::read_from_bytes(&anchor).unwrap();
            debug!("{v3:?}");
            major = v3.major;
            minor = v3.minor;
        } else {
            return None;
        }

        Some(Self {
            fwcfg,
            major,
            minor,
        })
    }

    pub fn version(&self) -> (u8, u8) {
        (self.major, self.minor)
    }

    pub fn tables(self) -> Option<Vec<u8>> {
        self.fwcfg.load_file("etc/smbios/smbios-tables")
    }
}
