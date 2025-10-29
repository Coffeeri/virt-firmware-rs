extern crate alloc;
use core::mem::size_of;
use core::ptr::addr_of;
use zerocopy::{FromBytes, Immutable, IntoBytes};

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

#[derive(Debug, FromBytes, IntoBytes, Immutable)]
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

#[derive(Debug, FromBytes, IntoBytes, Immutable)]
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
    anchor_item: u16,
    tables_item: u16,
    anchor_size: u32,
    tables_size: u32,
    version: Option<(u8, u8)>,
    anchor: Option<*const ()>,
}

impl<'f> SmbiosLoader<'f> {
    pub fn new(fwcfg: &'f dyn FwCfg) -> Option<Self> {
        let (anchor_item, anchor_size) = fwcfg.findfile("etc/smbios/smbios-anchor")?;
        let (tables_item, tables_size) = fwcfg.findfile("etc/smbios/smbios-tables")?;
        if anchor_size as usize != size_of::<SmbiosV2>()
            && anchor_size as usize != size_of::<SmbiosV3>()
        {
            return None;
        }
        Some(Self {
            fwcfg,
            anchor_item,
            tables_item,
            anchor_size,
            tables_size,
            version: None,
            anchor: None,
        })
    }

    pub fn load_tables<A>(&mut self, alloc_func: A) -> Result<(), &str>
    where
        A: Fn(usize, usize) -> Option<&'static mut [u8]>,
    {
        let anchor = alloc_func(self.anchor_size as usize, 64).ok_or("allocation failed")?;
        let tables = alloc_func(self.tables_size as usize, 64).ok_or("allocation failed")?;
        self.fwcfg.read_dma(
            Some(self.anchor_item),
            anchor.as_mut_ptr().cast(),
            self.anchor_size as usize,
        );
        self.fwcfg.read_dma(
            Some(self.tables_item),
            tables.as_mut_ptr().cast(),
            self.tables_size as usize,
        );

        if self.anchor_size as usize == size_of::<SmbiosV2>() {
            let (mut v2, _) = SmbiosV2::read_from_prefix(anchor).unwrap();
            v2.tableaddress = addr_of!(*tables).addr() as u32;

            let sum1: u32 = v2.as_bytes()[16..].iter().map(|b| *b as u32).sum();
            let fixup1 = (0u32.wrapping_sub(sum1) & 0xff) as u8;
            v2.ic = fixup1;

            let sum2: u32 = v2.as_bytes().iter().map(|b| *b as u32).sum();
            let fixup2 = (0u32.wrapping_sub(sum2) & 0xff) as u8;
            v2.checksum = fixup2;

            v2.write_to_prefix(anchor).unwrap();
            self.version = Some((v2.major, v2.minor));
        }

        if self.anchor_size as usize == size_of::<SmbiosV3>() {
            let (mut v3, _) = SmbiosV3::read_from_prefix(anchor).unwrap();
            v3.address = addr_of!(*tables).addr() as u64;

            let sum: u32 = v3.as_bytes().iter().map(|b| *b as u32).sum();
            let fixup = (0u32.wrapping_sub(sum) & 0xff) as u8;
            v3.checksum = fixup;

            v3.write_to_prefix(anchor).unwrap();
        }

        self.anchor = Some(anchor.as_ptr().cast());
        Ok(())
    }

    pub fn version(&self) -> Option<(u8, u8)> {
        self.version
    }

    pub fn anchor(&self) -> Option<*const ()> {
        self.anchor
    }
}
