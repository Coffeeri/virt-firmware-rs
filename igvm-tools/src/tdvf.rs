// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::ovmfmeta::{OVMF_META_LIST, OVMF_META_TDX_LIST};
use uguid::Guid;

const TABLE_HEADER_SIZE: usize = 18;
const HEADER_SIZE: usize = 16;
const ENTRY_SIZE: usize = 32;
const PAGE_SIZE: u64 = 4096;
const RESET_VECTOR: u64 = 0xffff_fff0;

/// A validated TDVF metadata table.
#[derive(Debug)]
pub struct Tdvf<'a> {
    pub sections: Vec<TdvfSection<'a>>,
}

/// A TDVF section and its borrowed firmware contents.
#[derive(Debug)]
pub struct TdvfSection<'a> {
    pub gpa: u64,
    pub memory_size: u64,
    pub kind: TdvfSectionType,
    pub measured: bool,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub(crate) struct TdvfRegion {
    pub file_offset: usize,
    pub raw_size: usize,
    pub gpa: u64,
    pub memory_size: u64,
    pub kind: TdvfSectionType,
    pub measured: bool,
}

/// A supported TDVF version 1 section type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TdvfSectionType {
    Bfv,
    Cfv,
    Hob,
    TempMemory,
}

fn read_u32(data: &[u8]) -> u32 {
    u32::from_le_bytes(data.try_into().expect("four-byte field"))
}

fn read_u64(data: &[u8]) -> u64 {
    u64::from_le_bytes(data.try_into().expect("eight-byte field"))
}

pub(crate) fn metadata_distance(firmware: &[u8]) -> Result<usize, String> {
    let footer_end = firmware
        .len()
        .checked_sub(32)
        .ok_or("firmware is too small for OVMF footer")?;
    let footer_start = footer_end
        .checked_sub(TABLE_HEADER_SIZE)
        .ok_or("truncated OVMF footer")?;
    if Guid::from_bytes(
        firmware[footer_start + 2..footer_end]
            .try_into()
            .expect("GUID size"),
    ) != OVMF_META_LIST
    {
        return Err("OVMF table footer not found".into());
    }
    let table_size = usize::from(u16::from_le_bytes(
        firmware[footer_start..footer_start + 2]
            .try_into()
            .expect("size field"),
    ));
    if table_size < TABLE_HEADER_SIZE {
        return Err("invalid OVMF table size".into());
    }
    let table_start = footer_end
        .checked_sub(table_size)
        .ok_or("OVMF table exceeds firmware")?;
    let mut entry_end = footer_start;
    let mut locator = None;
    while entry_end > table_start {
        let header = entry_end
            .checked_sub(TABLE_HEADER_SIZE)
            .filter(|&start| start >= table_start)
            .ok_or("truncated OVMF table entry")?;
        let entry_size = usize::from(u16::from_le_bytes(
            firmware[header..header + 2].try_into().expect("size field"),
        ));
        if entry_size < TABLE_HEADER_SIZE {
            return Err("invalid OVMF table entry size".into());
        }
        let entry_start = entry_end
            .checked_sub(entry_size)
            .filter(|&start| start >= table_start)
            .ok_or("OVMF table entry exceeds table")?;
        let guid = Guid::from_bytes(
            firmware[header + 2..entry_end]
                .try_into()
                .expect("GUID size"),
        );
        if guid == OVMF_META_TDX_LIST {
            if header - entry_start != 4 || locator.is_some() {
                return Err("invalid or duplicate TDVF metadata locator".into());
            }
            let distance = usize::try_from(read_u32(&firmware[entry_start..header]))
                .map_err(|_| "TDVF locator is too large")?;
            locator = Some(distance);
        }
        entry_end = entry_start;
    }
    locator.ok_or_else(|| "TDVF metadata locator not found".into())
}

pub(crate) fn metadata_offset(firmware: &[u8]) -> Result<usize, String> {
    firmware
        .len()
        .checked_sub(metadata_distance(firmware)?)
        .ok_or_else(|| "TDVF metadata locator exceeds firmware".into())
}

pub(crate) fn validate_bfv_metadata(firmware: &[u8]) -> Result<(), String> {
    let offset = metadata_offset(firmware)?;
    let regions = parse_descriptor(firmware, offset, firmware.len())?;
    let bfv = regions
        .iter()
        .find(|region| region.kind == TdvfSectionType::Bfv)
        .ok_or("missing BFV section")?;
    let raw_size = u64::try_from(bfv.raw_size).map_err(|_| "BFV size is too large")?;
    if raw_size != bfv.memory_size
        || bfv.gpa.checked_add(raw_size) != Some(0x1_0000_0000)
        || bfv.file_offset.checked_add(bfv.raw_size) != Some(firmware.len())
    {
        return Err("BFV raw data must end at 4 GiB and at the end of the firmware".into());
    }
    let descriptor_size = HEADER_SIZE
        .checked_add(
            regions
                .len()
                .checked_mul(ENTRY_SIZE)
                .ok_or("TDVF metadata length overflow")?,
        )
        .ok_or("TDVF metadata length overflow")?;
    let descriptor_end = offset
        .checked_add(descriptor_size)
        .ok_or("TDVF metadata offset overflow")?;
    let footer_end = firmware.len() - 32;
    let footer_start = footer_end - TABLE_HEADER_SIZE;
    let table_size = usize::from(u16::from_le_bytes(
        firmware[footer_start..footer_start + 2]
            .try_into()
            .expect("size field"),
    ));
    let table_start = footer_end - table_size;
    if offset < bfv.file_offset || descriptor_end > firmware.len() || table_start < bfv.file_offset
    {
        return Err("TDVF metadata and OVMF table must be contained in the BFV".into());
    }
    Ok(())
}

pub(crate) fn parse_descriptor(
    blob: &[u8],
    offset: usize,
    image_size: usize,
) -> Result<Vec<TdvfRegion>, String> {
    let header_end = offset
        .checked_add(HEADER_SIZE)
        .ok_or("TDVF header offset overflow")?;
    let header = blob
        .get(offset..header_end)
        .ok_or("truncated TDVF header")?;
    if &header[..4] != b"TDVF" {
        return Err("invalid TDVF signature".into());
    }
    if read_u32(&header[8..12]) != 1 {
        return Err("unsupported TDVF metadata version".into());
    }
    let length =
        usize::try_from(read_u32(&header[4..8])).map_err(|_| "TDVF length is too large")?;
    let count =
        usize::try_from(read_u32(&header[12..16])).map_err(|_| "TDVF entry count is too large")?;
    let expected = count
        .checked_mul(ENTRY_SIZE)
        .and_then(|size| size.checked_add(HEADER_SIZE))
        .ok_or("TDVF metadata length overflow")?;
    if length != expected {
        return Err("TDVF metadata length does not match entry count".into());
    }
    let end = offset
        .checked_add(length)
        .ok_or("TDVF metadata offset overflow")?;
    let entries = blob.get(header_end..end).ok_or("truncated TDVF metadata")?;
    let mut sections: Vec<TdvfRegion> = Vec::new();
    for (index, entry) in entries.chunks_exact(ENTRY_SIZE).enumerate() {
        let parse = || -> Result<TdvfRegion, String> {
            let raw_offset =
                usize::try_from(read_u32(&entry[..4])).map_err(|_| "raw offset is too large")?;
            let raw_size =
                usize::try_from(read_u32(&entry[4..8])).map_err(|_| "raw size is too large")?;
            let gpa = read_u64(&entry[8..16]);
            let memory_size = read_u64(&entry[16..24]);
            let kind = match read_u32(&entry[24..28]) {
                0 => TdvfSectionType::Bfv,
                1 => TdvfSectionType::Cfv,
                2 => TdvfSectionType::Hob,
                3 => TdvfSectionType::TempMemory,
                _ => return Err("unsupported section type".into()),
            };
            let attributes = read_u32(&entry[28..32]);
            if attributes & !1 != 0 || (kind == TdvfSectionType::Hob && attributes != 0) {
                return Err("unsupported section attributes".into());
            }
            if memory_size == 0
                || !memory_size.is_multiple_of(PAGE_SIZE)
                || !gpa.is_multiple_of(PAGE_SIZE)
            {
                return Err("memory address and nonzero size must be page aligned".into());
            }
            let end = gpa
                .checked_add(memory_size)
                .ok_or("memory range overflow")?;
            if sections
                .iter()
                .any(|section| gpa < section.gpa + section.memory_size && section.gpa < end)
            {
                return Err("memory range overlaps another entry".into());
            }
            if matches!(kind, TdvfSectionType::Bfv | TdvfSectionType::Cfv) != (raw_size != 0) {
                return Err("invalid raw data size for section type".into());
            }
            if u64::try_from(raw_size).map_err(|_| "raw size is too large")? > memory_size {
                return Err("raw data exceeds memory size".into());
            }
            let raw_end = raw_offset
                .checked_add(raw_size)
                .ok_or("raw data range overflow")?;
            if raw_end > image_size {
                return Err("raw data exceeds firmware".into());
            }
            Ok(TdvfRegion {
                file_offset: raw_offset,
                raw_size,
                gpa,
                memory_size,
                kind,
                measured: attributes == 1,
            })
        };
        sections.push(parse().map_err(|error| format!("TDVF entry {index}: {error}"))?);
    }
    let mut bfv = sections
        .iter()
        .filter(|section| section.kind == TdvfSectionType::Bfv);
    let boot = bfv.next().ok_or("missing BFV section")?;
    if bfv.next().is_some() || !(boot.gpa..boot.gpa + boot.memory_size).contains(&RESET_VECTOR) {
        return Err("exactly one BFV must cover the reset vector".into());
    }
    if sections
        .iter()
        .filter(|section| section.kind == TdvfSectionType::Hob)
        .count()
        != 1
    {
        return Err("exactly one HOB section is required".into());
    }
    Ok(sections)
}

impl Tdvf<'_> {
    /// Parses a firmware image and validates its TDVF version 1 metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the locator, metadata, or section ranges are invalid.
    pub fn parse(firmware: &[u8]) -> Result<Tdvf<'_>, String> {
        let offset = metadata_offset(firmware)?;
        let sections = parse_descriptor(firmware, offset, firmware.len())?
            .into_iter()
            .map(|region| TdvfSection {
                gpa: region.gpa,
                memory_size: region.memory_size,
                kind: region.kind,
                measured: region.measured,
                data: &firmware[region.file_offset..region.file_offset + region.raw_size],
            })
            .collect();
        Ok(Tdvf { sections })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA: usize = 64;
    const FIRST: usize = METADATA + HEADER_SIZE;
    const SECOND: usize = FIRST + ENTRY_SIZE;
    const LOCATOR: usize = 512 - 32 - TABLE_HEADER_SIZE - 22;
    const FOOTER: usize = 512 - 32 - TABLE_HEADER_SIZE;

    fn image() -> Vec<u8> {
        let mut data = vec![0u8; 512];
        data[..3].copy_from_slice(&[1, 2, 3]);
        data[METADATA..METADATA + 4].copy_from_slice(b"TDVF");
        data[METADATA + 4..METADATA + 8].copy_from_slice(&80u32.to_le_bytes());
        data[METADATA + 8..METADATA + 12].copy_from_slice(&1u32.to_le_bytes());
        data[METADATA + 12..METADATA + 16].copy_from_slice(&2u32.to_le_bytes());
        data[FIRST + 4..FIRST + 8].copy_from_slice(&3u32.to_le_bytes());
        data[FIRST + 8..FIRST + 16].copy_from_slice(&0xffff_f000u64.to_le_bytes());
        data[FIRST + 16..FIRST + 24].copy_from_slice(&0x1000u64.to_le_bytes());
        data[FIRST + 28..FIRST + 32].copy_from_slice(&1u32.to_le_bytes());
        data[SECOND + 8..SECOND + 16].copy_from_slice(&0x1000u64.to_le_bytes());
        data[SECOND + 16..SECOND + 24].copy_from_slice(&0x1000u64.to_le_bytes());
        data[SECOND + 24..SECOND + 28].copy_from_slice(&2u32.to_le_bytes());
        data[LOCATOR..LOCATOR + 4].copy_from_slice(&448u32.to_le_bytes());
        data[LOCATOR + 4..LOCATOR + 6].copy_from_slice(&22u16.to_le_bytes());
        data[LOCATOR + 6..FOOTER].copy_from_slice(&OVMF_META_TDX_LIST.to_bytes());
        data[FOOTER..FOOTER + 2].copy_from_slice(&40u16.to_le_bytes());
        data[FOOTER + 2..FOOTER + 18].copy_from_slice(&OVMF_META_LIST.to_bytes());
        data
    }

    fn rejects(offset: usize, value: &[u8], message: &str) {
        let mut firmware = image();
        firmware[offset..offset + value.len()].copy_from_slice(value);
        let error = Tdvf::parse(&firmware).unwrap_err();
        assert!(error.contains(message), "{error}");
    }

    #[test]
    fn parses_sections_in_order_and_borrows_data() {
        let firmware = image();
        let tdvf = Tdvf::parse(&firmware).unwrap();
        assert_eq!(tdvf.sections.len(), 2);
        assert_eq!(tdvf.sections[0].kind, TdvfSectionType::Bfv);
        assert_eq!(tdvf.sections[0].data, &firmware[..3]);
        assert!(tdvf.sections[0].measured);
        assert_eq!(tdvf.sections[1].kind, TdvfSectionType::Hob);
        assert!(!tdvf.sections[1].measured);
    }

    #[test]
    fn rejects_metadata_outside_bfv() {
        assert!(validate_bfv_metadata(&image()).is_err());
        let mut firmware = vec![0u8; 4096];
        firmware[4096 - 512..].copy_from_slice(&image());
        let entry = 4096 - 512 + FIRST;
        firmware[entry + 4..entry + 8].copy_from_slice(&4096u32.to_le_bytes());
        assert!(validate_bfv_metadata(&firmware).is_ok());
    }

    #[test]
    fn parses_descriptor_from_tail_with_original_image_bounds() {
        let firmware = image();
        let tail = &firmware[METADATA..];
        let regions = parse_descriptor(tail, 0, firmware.len()).unwrap();
        assert_eq!(regions[0].file_offset, 0);
        assert_eq!(regions[0].raw_size, 3);
        assert!(parse_descriptor(tail, 0, 2).is_err());
        assert!(parse_descriptor(&tail[..HEADER_SIZE], 0, firmware.len()).is_err());
    }

    #[test]
    fn rejects_invalid_table_and_header() {
        assert!(Tdvf::parse(&[]).is_err());
        rejects(FOOTER + 2, &[0], "footer");
        rejects(LOCATOR + 4, &1u16.to_le_bytes(), "entry size");
        rejects(LOCATOR, &u32::MAX.to_le_bytes(), "locator exceeds");
        rejects(METADATA, &[0], "signature");
        rejects(METADATA + 4, &79u32.to_le_bytes(), "length");
        rejects(METADATA + 8, &2u32.to_le_bytes(), "version");
        rejects(METADATA + 12, &u32::MAX.to_le_bytes(), "length");
    }

    #[test]
    fn rejects_invalid_sections() {
        rejects(FIRST + 8, &1u64.to_le_bytes(), "aligned");
        rejects(FIRST + 8, &(u64::MAX - 4095).to_le_bytes(), "overflow");
        rejects(SECOND + 8, &0xffff_f000u64.to_le_bytes(), "overlaps");
        rejects(FIRST, &511u32.to_le_bytes(), "exceeds firmware");
        rejects(FIRST + 4, &4097u32.to_le_bytes(), "exceeds memory");
        rejects(FIRST + 4, &0u32.to_le_bytes(), "raw data size");
        rejects(SECOND + 4, &1u32.to_le_bytes(), "raw data size");
        rejects(SECOND + 24, &4u32.to_le_bytes(), "section type");
        rejects(FIRST + 28, &2u32.to_le_bytes(), "attributes");
        rejects(SECOND + 28, &1u32.to_le_bytes(), "attributes");
        rejects(FIRST + 8, &0x2000u64.to_le_bytes(), "reset vector");
        rejects(SECOND + 24, &3u32.to_le_bytes(), "HOB");
    }
}
