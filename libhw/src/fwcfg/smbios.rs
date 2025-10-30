extern crate alloc;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::addr_of;
use log::debug;
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

/*
  SMBIOS_TYPE      Type;
  UINT8            Length;
  SMBIOS_HANDLE    Handle;
  ---
  SMBIOS_STRUCTURE             Hdr;
  SMBIOS_TABLE_STRING          Vendor;
  SMBIOS_TABLE_STRING          BiosVersion;
  UINT16                       BiosSegment;
  SMBIOS_TABLE_STRING          BiosReleaseDate;
  UINT8                        BiosSize;
  MISC_BIOS_CHARACTERISTICS    BiosCharacteristics;
  UINT8                        BIOSCharacteristicsExtensionBytes[2];
  UINT8                        SystemBiosMajorRelease;
  UINT8                        SystemBiosMinorRelease;
  UINT8                        EmbeddedControllerFirmwareMajorRelease;
  UINT8                        EmbeddedControllerFirmwareMinorRelease;
*/

#[derive(Debug, FromBytes, IntoBytes, Immutable)]
#[repr(C, packed)]
struct SmbiosType0 {
    // header
    table_type: u8,
    length: u8,
    handle: u16,
    // type 0
    vendor: u8,       // string
    bios_version: u8, // string
    bios_segment: u16,
    bios_date: u8, // string
    bios_size: u8,
    bios_characteristics: [u32; 2],
    bios_characteristics_ext: [u8; 2],
    bios_major: u8,
    bios_minor: u8,
    ec_major: u8,
    ec_minor: u8,
}

pub struct SmbiosLoader<'f> {
    fwcfg: &'f dyn FwCfg,
    anchor_item: u16,
    tables_item: u16,
    anchor_size: u32,
    tables_size: u32,
    version: Option<(u8, u8)>,
    anchor: Option<*const ()>,
    // table 0 info
    bios_major: u8,
    bios_minor: u8,
    bios_vendor: Option<&'f str>,
    bios_version: Option<&'f str>,
    bios_date: Option<&'f str>,
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
            bios_major: 0,
            bios_minor: 0,
            bios_vendor: None,
            bios_version: None,
            bios_date: None,
        })
    }

    pub fn set_bios(
        &mut self,
        major: u8,
        minor: u8,
        vendor: &'f str,
        version: &'f str,
        date: &'f str,
    ) {
        self.bios_major = major;
        self.bios_minor = minor;
        self.bios_vendor = Some(vendor);
        self.bios_version = Some(version);
        self.bios_date = Some(date);
    }

    fn type0(&self) -> Option<Vec<u8>> {
        let mut blob = Vec::new();
        let zero: [u8; 1] = [0];
        let characteristics = 1 << 3; // unsupported
        let ext1 = 1 << 0; // ACPI
        let ext2 = (1 << 3)  // UEFI
            | (1 << 4); // virtuaL machine
        let type0 = SmbiosType0 {
            table_type: 0,
            length: size_of::<SmbiosType0>() as u8,
            handle: 0,
            vendor: 1,
            bios_version: 2,
            bios_segment: 0,
            bios_date: 3,
            bios_size: 0,
            bios_characteristics: [characteristics, 0],
            bios_characteristics_ext: [ext1, ext2],
            bios_major: self.bios_major,
            bios_minor: self.bios_minor,
            ec_major: self.bios_major,
            ec_minor: self.bios_minor,
        };
        blob.extend_from_slice(type0.as_bytes());
        blob.extend_from_slice(self.bios_vendor?.as_bytes());
        blob.extend_from_slice(&zero);
        blob.extend_from_slice(self.bios_version?.as_bytes());
        blob.extend_from_slice(&zero);
        blob.extend_from_slice(self.bios_date?.as_bytes());
        blob.extend_from_slice(&zero);
        blob.extend_from_slice(&zero);
        Some(blob)
    }

    pub fn load_tables<A>(&mut self, alloc_func: A) -> Result<(), &str>
    where
        A: Fn(usize, usize) -> Option<&'static mut [u8]>,
    {
        let anchor = alloc_func(self.anchor_size as usize, 64).ok_or("allocation failed")?;
        self.fwcfg.read_dma(
            Some(self.anchor_item),
            anchor.as_mut_ptr().cast(),
            self.anchor_size as usize,
        );

        let type0 = self.type0().unwrap_or_default();
        let tables =
            alloc_func(self.tables_size as usize + type0.len(), 64).ok_or("allocation failed")?;
        tables[..type0.len()].copy_from_slice(&type0);
        self.fwcfg.read_dma(
            Some(self.tables_item),
            tables[type0.len()..].as_mut_ptr().cast(),
            self.tables_size as usize,
        );

        if self.anchor_size as usize == size_of::<SmbiosV2>() {
            let (mut v2, _) = SmbiosV2::read_from_prefix(anchor).unwrap();
            if !type0.is_empty() {
                let tl = v2.tablelength;
                debug!("smbios2: qemu {}, type0 {}", tl, type0.len());
                v2.tablelength += type0.len() as u16;
                v2.count += 1;
            }
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
            if !type0.is_empty() {
                let tl = v3.maxsize;
                debug!("smbios3: qemu {}, type0 {}", tl, type0.len());
                v3.maxsize += type0.len() as u32;
            }
            v3.address = addr_of!(*tables).addr() as u64;

            let sum: u32 = v3.as_bytes().iter().map(|b| *b as u32).sum();
            let fixup = (0u32.wrapping_sub(sum) & 0xff) as u8;
            v3.checksum = fixup;

            v3.write_to_prefix(anchor).unwrap();
            self.version = Some((v3.major, v3.minor));
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
