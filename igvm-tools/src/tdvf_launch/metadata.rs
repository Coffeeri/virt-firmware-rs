use std::collections::BTreeMap;

use igvm::{IgvmDirectiveHeader, IgvmFile};
use igvm_defs::IgvmPageDataType;

use super::{ImportRanges, PAGE_SIZE};
use crate::tdvf::{metadata_distance, parse_descriptor, TdvfRegion, TdvfSectionType};

const FIRMWARE_END: u64 = 1u64 << 32;

fn selected(directive: &IgvmDirectiveHeader, mask: u32) -> bool {
    directive
        .compatibility_mask()
        .is_none_or(|compat| compat & mask != 0)
}

fn read_pages(
    pages: &BTreeMap<u64, &Vec<u8>>,
    start: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    let length_u64 = u64::try_from(length).map_err(|_| "TDVF metadata length overflow")?;
    let end = start
        .checked_add(length_u64)
        .ok_or("TDVF metadata range overflow")?;
    let available = pages
        .len()
        .checked_mul(PAGE_SIZE as usize)
        .ok_or("TDVF page count overflow")?;
    if length > available || end > FIRMWARE_END {
        return Err("TDVF metadata exceeds available firmware pages".into());
    }
    let mut result = Vec::new();
    let mut address = start;
    while address < end {
        let page_gpa = address & !(PAGE_SIZE - 1);
        let data = pages
            .get(&page_gpa)
            .ok_or("missing page containing TDVF metadata")?;
        let offset = (address - page_gpa) as usize;
        let count = ((end - address).min(PAGE_SIZE - offset as u64)) as usize;
        let output_offset = result.len();
        result.resize(output_offset + count, 0);
        if let Some(bytes) = data.get(offset..) {
            let copy_length = bytes.len().min(count);
            result[output_offset..output_offset + copy_length]
                .copy_from_slice(&bytes[..copy_length]);
        }
        address += count as u64;
    }
    Ok(result)
}

fn embedded_metadata(file: &IgvmFile, mask: u32) -> Result<Vec<TdvfRegion>, String> {
    let mut pages = BTreeMap::new();
    for directive in file
        .directives()
        .iter()
        .filter(|directive| selected(directive, mask))
    {
        if let IgvmDirectiveHeader::PageData {
            gpa,
            flags,
            data_type,
            data,
            ..
        } = directive
        {
            if !gpa.is_multiple_of(PAGE_SIZE)
                || flags.shared()
                || flags.is_2mb_page()
                || flags.reserved() != 0
                || *data_type != IgvmPageDataType::NORMAL
                || data.len() > PAGE_SIZE as usize
                || pages.insert(*gpa, data).is_some()
            {
                return Err("unsupported or duplicate TDVF page".into());
            }
        }
    }
    // Only the architectural reset window is fixed. All section addresses,
    // including the HOB destination, come from the embedded descriptor.
    let footer = read_pages(&pages, FIRMWARE_END - 50, 18)?;
    let table_size = u16::from_le_bytes(footer[..2].try_into().expect("footer size")) as u64;
    let table_gpa = FIRMWARE_END
        .checked_sub(32 + table_size)
        .ok_or("OVMF table range overflow")?;
    let table = read_pages(&pages, table_gpa, usize::from(table_size as u16) + 32)?;
    let distance = metadata_distance(&table)?;
    let descriptor_gpa = FIRMWARE_END
        .checked_sub(distance as u64)
        .ok_or("TDVF descriptor address overflow")?;
    let header = read_pages(&pages, descriptor_gpa, 16)?;
    let descriptor_size = u32::from_le_bytes(header[4..8].try_into().expect("descriptor length"));
    let descriptor = read_pages(&pages, descriptor_gpa, descriptor_size as usize)?;
    // Raw offsets refer to the original FD, not to the descriptor buffer.
    let regions = parse_descriptor(&descriptor, 0, usize::MAX)?;
    let bfv = regions
        .iter()
        .find(|region| region.kind == TdvfSectionType::Bfv)
        .ok_or("missing BFV")?;
    if bfv.gpa.checked_add(bfv.memory_size) != Some(FIRMWARE_END)
        || bfv.memory_size != bfv.raw_size as u64
    {
        return Err("TDVF profile requires the raw BFV to end at the x86 reset window".into());
    }
    let image_size = bfv
        .file_offset
        .checked_add(bfv.raw_size)
        .ok_or("BFV file range overflow")?;
    if regions.iter().any(|region| {
        region
            .file_offset
            .checked_add(region.raw_size)
            .is_none_or(|end| end > image_size)
    }) {
        return Err("TDVF raw section exceeds original firmware image".into());
    }
    let descriptor_end = descriptor_gpa
        .checked_add(descriptor_size as u64)
        .ok_or("descriptor size overflow")?;
    if descriptor_gpa < bfv.gpa || descriptor_end > FIRMWARE_END || table_gpa < bfv.gpa {
        return Err("TDVF descriptor and GUID table must be contained in the BFV".into());
    }
    Ok(regions)
}

pub(super) fn resolve_imports(file: &IgvmFile, mask: u32) -> Result<ImportRanges, String> {
    let regions = embedded_metadata(file, mask)?;
    let hob = regions
        .iter()
        .find(|region| region.kind == TdvfSectionType::Hob)
        .ok_or("missing TD HOB metadata")?;
    let mut areas = BTreeMap::new();
    for directive in file
        .directives()
        .iter()
        .filter(|directive| selected(directive, mask))
    {
        match directive {
            IgvmDirectiveHeader::ParameterArea {
                parameter_area_index,
                number_of_bytes,
                initial_data,
            } => {
                if !initial_data.is_empty()
                    || areas
                        .insert(*parameter_area_index, *number_of_bytes)
                        .is_some()
                {
                    return Err("invalid TDVF parameter area".into());
                }
            }
            IgvmDirectiveHeader::PageData { .. } | IgvmDirectiveHeader::ParameterInsert(_) => {}
            _ => return Err("unsupported TDVF directive".into()),
        }
    }
    let mut imports = file
        .directives()
        .iter()
        .filter(|directive| selected(directive, mask))
        .filter(|directive| {
            matches!(
                directive,
                IgvmDirectiveHeader::PageData { .. } | IgvmDirectiveHeader::ParameterInsert(_)
            )
        });
    let mut bound_area = None;
    for region in &regions {
        if region.kind == TdvfSectionType::Hob {
            match imports.next() {
                Some(IgvmDirectiveHeader::ParameterInsert(insert))
                    if insert.gpa == hob.gpa
                        && areas.get(&insert.parameter_area_index) == Some(&hob.memory_size) =>
                {
                    bound_area = Some(insert.parameter_area_index);
                }
                _ => return Err("TD HOB insertion disagrees with embedded TDVF metadata".into()),
            }
            continue;
        }
        for offset in (0..region.memory_size).step_by(PAGE_SIZE as usize) {
            match imports.next() {
                Some(IgvmDirectiveHeader::PageData {
                    gpa, flags, data, ..
                }) if *gpa == region.gpa + offset && flags.unmeasured() != region.measured => {
                    let raw_bytes = region
                        .raw_size
                        .saturating_sub(
                            usize::try_from(offset)
                                .map_err(|_| "TDVF offset exceeds address space")?,
                        )
                        .min(PAGE_SIZE as usize);
                    if data
                        .get(raw_bytes..)
                        .unwrap_or_default()
                        .iter()
                        .any(|byte| *byte != 0)
                    {
                        return Err("TDVF zero-fill disagrees with embedded metadata".into());
                    }
                }
                _ => return Err(
                    "TDVF import order, address or measurement disagrees with embedded metadata"
                        .into(),
                ),
            }
        }
    }
    if imports.next().is_some() || areas.len() != 1 || bound_area.is_none() {
        return Err("unexpected TDVF imports or parameter areas".into());
    }
    let mut imports: Vec<_> = regions
        .iter()
        .map(|region| (region.gpa, region.gpa + region.memory_size))
        .collect();
    imports.sort_unstable();
    Ok(ImportRanges {
        gpa: hob.gpa,
        size: hob.memory_size,
        imports,
    })
}
