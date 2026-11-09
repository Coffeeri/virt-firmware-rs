//! Serialization of TDVF hand-off blocks supplied by a VMM.

/// The type of a TDVF memory resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceType {
    /// Accepted system memory.
    Accepted,
    /// Memory that the guest must accept before use.
    Unaccepted,
    /// Reserved memory.
    Reserved,
}

/// A page-aligned guest physical memory range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryResource {
    /// Start address of the range.
    pub start: u64,
    /// Length in bytes.
    pub length: u64,
    /// Resource type.
    pub kind: ResourceType,
}

const ACPI_GUID: [u8; 16] = [
    0x70, 0x58, 0x0c, 0x6a, 0xed, 0xd4, 0xf4, 0x44, 0xa1, 0x35, 0xdd, 0x23, 0x8b, 0x6f, 0x0c, 0x8d,
];

fn append_hob(
    data: &mut Vec<u8>,
    capacity: usize,
    kind: u16,
    payload: &[u8],
) -> Result<(), String> {
    let length = 8usize
        .checked_add(payload.len())
        .ok_or("HOB length overflow")?;
    let padded = length.checked_add(7).ok_or("HOB length overflow")? & !7;
    let hob_length = u16::try_from(padded).map_err(|_| "HOB exceeds u16 length")?;
    let end = data
        .len()
        .checked_add(padded)
        .ok_or("HOB list length overflow")?;
    if end > capacity {
        return Err("HOB list exceeds capacity".into());
    }
    data.extend_from_slice(&kind.to_le_bytes());
    data.extend_from_slice(&hob_length.to_le_bytes());
    data.extend_from_slice(&[0; 4]);
    data.extend_from_slice(payload);
    data.resize(end, 0);
    Ok(())
}

/// Builds a zero-padded TDVF HOB list in a guest physical memory region.
///
/// Memory ranges must be nonempty, page-aligned, and disjoint. ACPI tables
/// must have valid lengths and checksums and contain exactly one DSDT.
///
/// # Errors
///
/// Returns an error if an input is invalid or the list does not fit.
pub fn build_tdvf_hob(
    gpa: u64,
    capacity: usize,
    resources: &[MemoryResource],
    acpi_tables: &[&[u8]],
) -> Result<Vec<u8>, String> {
    if !gpa.is_multiple_of(8) {
        return Err("HOB address is not 8-byte aligned".into());
    }
    let capacity_u64 = u64::try_from(capacity).map_err(|_| "capacity exceeds u64")?;
    gpa.checked_add(capacity_u64)
        .ok_or("HOB region overflows guest address space")?;

    let mut ranges = Vec::with_capacity(resources.len());
    for resource in resources {
        if resource.length == 0
            || !resource.start.is_multiple_of(4096)
            || !resource.length.is_multiple_of(4096)
        {
            return Err("memory resource must be nonempty and page-aligned".into());
        }
        let end = resource
            .start
            .checked_add(resource.length)
            .ok_or("memory resource overflows")?;
        ranges.push((resource.start, end));
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("memory resources overlap".into());
    }

    let mut dsdt_count = 0;
    for table in acpi_tables {
        if table.len() < 36 {
            return Err("ACPI table header is truncated".into());
        }
        let length = u32::from_le_bytes([table[4], table[5], table[6], table[7]]);
        if usize::try_from(length).ok() != Some(table.len()) {
            return Err("ACPI table length does not match payload".into());
        }
        if table.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) != 0 {
            return Err("ACPI table checksum is invalid".into());
        }
        dsdt_count += usize::from(&table[..4] == b"DSDT");
    }
    if dsdt_count != 1 {
        return Err("exactly one DSDT is required".into());
    }

    let mut data = Vec::new();
    data.try_reserve_exact(capacity)
        .map_err(|_| "cannot allocate HOB region")?;
    let mut phit = [0u8; 48];
    phit[..4].copy_from_slice(&9u32.to_le_bytes());
    append_hob(&mut data, capacity, 1, &phit)?;
    const PRESENT: u32 = 0x1;
    const INITIALIZED: u32 = 0x2;
    const TESTED: u32 = 0x4;
    for resource in resources {
        // Reserved ranges are not marked tested; QEMU does not emit them here.
        let (kind, attributes): (u32, u32) = match resource.kind {
            ResourceType::Accepted => (0, PRESENT | INITIALIZED | TESTED),
            ResourceType::Unaccepted => (7, PRESENT | INITIALIZED | TESTED),
            ResourceType::Reserved => (5, PRESENT | INITIALIZED),
        };
        let mut descriptor = [0u8; 40];
        descriptor[16..20].copy_from_slice(&kind.to_le_bytes());
        descriptor[20..24].copy_from_slice(&attributes.to_le_bytes());
        descriptor[24..32].copy_from_slice(&resource.start.to_le_bytes());
        descriptor[32..40].copy_from_slice(&resource.length.to_le_bytes());
        append_hob(&mut data, capacity, 3, &descriptor)?;
    }
    for table in acpi_tables {
        let payload_len = 16usize
            .checked_add(table.len())
            .ok_or("HOB length overflow")?;
        let padded = payload_len.checked_add(15).ok_or("HOB length overflow")? & !7;
        if padded > usize::from(u16::MAX) || padded > capacity.saturating_sub(data.len()) {
            return Err("ACPI HOB exceeds size limit or capacity".into());
        }
        let mut payload = Vec::with_capacity(payload_len);
        payload.extend_from_slice(&ACPI_GUID);
        payload.extend_from_slice(table);
        append_hob(&mut data, capacity, 4, &payload)?;
    }
    let end_address = gpa
        .checked_add(u64::try_from(data.len()).map_err(|_| "HOB list exceeds u64")?)
        .ok_or("HOB end address overflows")?;
    append_hob(&mut data, capacity, 0xffff, &[])?;
    data[48..56].copy_from_slice(&end_address.to_le_bytes());
    data.resize(capacity, 0);
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(signature: &[u8; 4], extra: usize) -> Vec<u8> {
        let mut data = vec![0u8; 36 + extra];
        data[..4].copy_from_slice(signature);
        let length = u32::try_from(data.len()).unwrap();
        data[4..8].copy_from_slice(&length.to_le_bytes());
        data[9] = 0u8.wrapping_sub(data.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)));
        data
    }

    fn u16_at(data: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap())
    }

    fn u32_at(data: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
    }

    fn u64_at(data: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
    }

    #[test]
    fn serializes_and_pads_hobs() {
        let dsdt = table(b"DSDT", 1);
        let resources = [
            MemoryResource {
                start: 0x2000,
                length: 0x1000,
                kind: ResourceType::Unaccepted,
            },
            MemoryResource {
                start: 0,
                length: 0x1000,
                kind: ResourceType::Accepted,
            },
            MemoryResource {
                start: 0x3000,
                length: 0x1000,
                kind: ResourceType::Reserved,
            },
        ];
        let data = build_tdvf_hob(0x100000, 288, &resources, &[&dsdt]).unwrap();
        assert_eq!(data.len(), 288);
        assert_eq!(
            (u16_at(&data, 0), u16_at(&data, 2), u32_at(&data, 8)),
            (1, 56, 9)
        );
        assert!(data[12..48].iter().all(|byte| *byte == 0));
        for (index, (kind, attributes, start)) in [(7, 0x7, 0x2000), (0, 0x7, 0), (5, 0x3, 0x3000)]
            .iter()
            .enumerate()
        {
            let offset = 56 + index * 48;
            assert_eq!((u16_at(&data, offset), u16_at(&data, offset + 2)), (3, 48));
            assert!(data[offset + 8..offset + 24].iter().all(|byte| *byte == 0));
            assert_eq!(
                (u32_at(&data, offset + 24), u32_at(&data, offset + 28)),
                (*kind, *attributes)
            );
            assert_eq!(
                (u64_at(&data, offset + 32), u64_at(&data, offset + 40)),
                (*start, 0x1000)
            );
        }
        let offset = 200;
        assert_eq!((u16_at(&data, offset), u16_at(&data, offset + 2)), (4, 64));
        assert_eq!(&data[offset + 8..offset + 24], &ACPI_GUID);
        assert_eq!(&data[offset + 24..offset + 61], &dsdt);
        assert_eq!(u64_at(&data, 48), 0x100000 + 264);
        assert_eq!(u16_at(&data, 264), 0xffff);
    }

    #[test]
    fn rejects_invalid_inputs() {
        let dsdt = table(b"DSDT", 0);
        assert!(build_tdvf_hob(0, 127, &[], &[&dsdt]).is_err());
        assert!(build_tdvf_hob(u64::MAX - 7, 128, &[], &[&dsdt]).is_err());
        assert!(build_tdvf_hob(0, 128, &[], &[]).is_err());
        assert!(build_tdvf_hob(0, 256, &[], &[&dsdt, &dsdt]).is_err());
        let mut bad = dsdt.clone();
        bad[10] = 1;
        assert!(build_tdvf_hob(0, 256, &[], &[&bad]).is_err());
        assert!(build_tdvf_hob(0, 256, &[], &[&dsdt[..20]]).is_err());
        let resource = MemoryResource {
            start: 0,
            length: 4096,
            kind: ResourceType::Accepted,
        };
        assert!(build_tdvf_hob(0, 256, &[resource, resource], &[&dsdt]).is_err());
        assert!(build_tdvf_hob(
            0,
            256,
            &[MemoryResource {
                start: u64::MAX - 4095,
                ..resource
            }],
            &[&dsdt]
        )
        .is_err());
        assert!(build_tdvf_hob(0, 70000, &[], &[&table(b"DSDT", 65500)]).is_err());
    }
}
