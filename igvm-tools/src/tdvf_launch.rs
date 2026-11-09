//! Preparation of ordered TDVF page imports for a TDX loader.

use igvm::{IgvmDirectiveHeader, IgvmFile, IgvmInitializationHeader, IgvmPlatformHeader};
use igvm_defs::IgvmPlatformType;

use crate::tdvf_hob::{build_tdvf_hob, MemoryResource, ResourceType};

mod metadata;

const PAGE_SIZE: u64 = 4096;

/// One private 4 KiB page to add to the TD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdvfPage {
    /// Guest physical address of the page.
    pub gpa: u64,
    /// Whether the page contents participate in the launch measurement.
    pub measured: bool,
    /// Page contents, padded to 4 KiB.
    pub data: Vec<u8>,
}

/// Ordered TDVF imports and the initial value for the boot vCPU's RCX.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdvfLaunch {
    /// Guest physical address of the TD-HOB list.
    pub initial_rcx: u64,
    /// Pages in the order in which the loader must import them.
    pub pages: Vec<TdvfPage>,
}

/// Prepares a TDVF launch from an IGVM file and the runtime machine topology.
///
/// The selected profile must retain its embedded TDVF metadata. Resources
/// describe initially unaccepted RAM and reserved ranges; imports within RAM
/// are marked accepted in the generated HOB. The loader must import every page
/// in the returned order before starting the guest.
///
/// # Errors
///
/// Returns an error for unsupported directives, invalid ranges or an invalid
/// HOB insertion, or if the generated HOB does not fit its parameter area.
pub fn prepare_tdvf_launch(
    file: &IgvmFile,
    compatibility_mask: u32,
    resources: &[MemoryResource],
    acpi_tables: &[&[u8]],
) -> Result<TdvfLaunch, String> {
    validate_profile(file, compatibility_mask)?;
    let ImportRanges { gpa, size, imports } = metadata::resolve_imports(file, compatibility_mask)?;
    let resources = classify_resources(resources, &imports, gpa, size)?;
    let capacity = usize::try_from(size).map_err(|_| "HOB area exceeds host address space")?;
    let hob = build_tdvf_hob(gpa, capacity, &resources, acpi_tables)?;
    let pages = render_imports(file, compatibility_mask, gpa, &hob);
    Ok(TdvfLaunch {
        initial_rcx: gpa,
        pages,
    })
}

fn validate_profile(file: &IgvmFile, compatibility_mask: u32) -> Result<(), String> {
    if !compatibility_mask.is_power_of_two()
        || !file.platforms().iter().any(|platform| {
            let IgvmPlatformHeader::SupportedPlatform(info) = platform;
            info.compatibility_mask == compatibility_mask
                && info.platform_type == IgvmPlatformType::TDX
                && info.highest_vtl == 0
        })
    {
        return Err("compatibility mask does not select a TDX profile".into());
    }
    for init in file.initializations() {
        let mask = match init {
            IgvmInitializationHeader::GuestPolicy {
                compatibility_mask, ..
            }
            | IgvmInitializationHeader::CcaPolicy {
                compatibility_mask, ..
            }
            | IgvmInitializationHeader::RelocatableRegion {
                compatibility_mask, ..
            }
            | IgvmInitializationHeader::PageTableRelocationRegion {
                compatibility_mask, ..
            } => *compatibility_mask,
        };
        if mask & compatibility_mask != 0 {
            return Err("unsupported initialization header".into());
        }
    }
    Ok(())
}

struct ImportRanges {
    gpa: u64,
    size: u64,
    imports: Vec<(u64, u64)>,
}

fn classify_resources(
    resources: &[MemoryResource],
    imports: &[(u64, u64)],
    gpa: u64,
    size: u64,
) -> Result<Vec<MemoryResource>, String> {
    let end = gpa.checked_add(size).ok_or("HOB range overflow")?;
    let mut baseline = Vec::with_capacity(resources.len());
    for resource in resources {
        if resource.kind == ResourceType::Accepted
            || resource.length == 0
            || !resource.start.is_multiple_of(PAGE_SIZE)
            || !resource.length.is_multiple_of(PAGE_SIZE)
        {
            return Err("resources must be page-aligned unaccepted or reserved ranges".into());
        }
        baseline.push((
            resource.start,
            resource
                .start
                .checked_add(resource.length)
                .ok_or("resource range overflow")?,
            resource.kind,
        ));
    }
    baseline.sort_unstable_by_key(|range| range.0);
    if baseline.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("memory resources overlap".into());
    }
    if !baseline.iter().any(|(start, limit, kind)| {
        *kind == ResourceType::Unaccepted && *start <= gpa && end <= *limit
    }) {
        return Err("HOB area must lie within unaccepted RAM".into());
    }
    if baseline.iter().any(|(start, end, kind)| {
        *kind == ResourceType::Reserved && imports.iter().any(|(from, to)| from < end && start < to)
    }) {
        return Err("TDVF import overlaps reserved memory".into());
    }
    let mut classified = Vec::new();
    for (start, limit, kind) in baseline {
        let mut cursor = start;
        if kind == ResourceType::Unaccepted {
            for &(import_start, import_end) in imports {
                let from = import_start.max(start);
                let to = import_end.min(limit);
                if from >= to {
                    continue;
                }
                if cursor < from {
                    classified.push(MemoryResource {
                        start: cursor,
                        length: from - cursor,
                        kind,
                    });
                }
                classified.push(MemoryResource {
                    start: from,
                    length: to - from,
                    kind: ResourceType::Accepted,
                });
                cursor = to;
            }
        }
        if cursor < limit {
            classified.push(MemoryResource {
                start: cursor,
                length: limit - cursor,
                kind,
            });
        }
    }
    let mut merged: Vec<MemoryResource> = Vec::new();
    for resource in classified {
        if let Some(previous) = merged.last_mut() {
            if previous.kind == resource.kind && previous.start + previous.length == resource.start
            {
                previous.length += resource.length;
                continue;
            }
        }
        merged.push(resource);
    }
    Ok(merged)
}

fn render_imports(file: &IgvmFile, compatibility_mask: u32, gpa: u64, hob: &[u8]) -> Vec<TdvfPage> {
    let mut pages = Vec::new();
    for directive in file.directives() {
        if directive
            .compatibility_mask()
            .is_some_and(|mask| mask & compatibility_mask == 0)
        {
            continue;
        }
        match directive {
            IgvmDirectiveHeader::PageData {
                gpa, flags, data, ..
            } => {
                let mut page = data.clone();
                page.resize(PAGE_SIZE as usize, 0);
                pages.push(TdvfPage {
                    gpa: *gpa,
                    measured: !flags.unmeasured(),
                    data: page,
                });
            }
            IgvmDirectiveHeader::ParameterInsert(_) => {
                for (index, chunk) in hob.chunks_exact(PAGE_SIZE as usize).enumerate() {
                    pages.push(TdvfPage {
                        gpa: gpa + index as u64 * PAGE_SIZE,
                        measured: false,
                        data: chunk.to_vec(),
                    });
                }
            }
            _ => {}
        }
    }
    pages
}
