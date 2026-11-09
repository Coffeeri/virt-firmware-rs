use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use igvm::{Arch, IgvmDirectiveHeader, IgvmFile, IgvmPlatformHeader, IgvmRevision};
use igvm_defs::{IgvmPageDataType, IgvmPlatformType};
use sha2::{Digest, Sha384};
use virtfw_igvm_tools::ovmfmeta::{OVMF_META_LIST, OVMF_META_TDX_LIST};
use virtfw_igvm_tools::tdvf::Tdvf;
use virtfw_igvm_tools::tdvf_hob::{MemoryResource, ResourceType};
use virtfw_igvm_tools::tdvf_launch::prepare_tdvf_launch;

const PAGE: usize = 4096;
static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tdvf-wrap-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn firmware() -> Vec<u8> {
    let mut data = vec![0u8; PAGE];
    data[..3].copy_from_slice(&[1, 2, 3]);
    let metadata = 64;
    data[metadata..metadata + 4].copy_from_slice(b"TDVF");
    data[metadata + 4..metadata + 8].copy_from_slice(&112u32.to_le_bytes());
    data[metadata + 8..metadata + 12].copy_from_slice(&1u32.to_le_bytes());
    data[metadata + 12..metadata + 16].copy_from_slice(&3u32.to_le_bytes());
    let bfv = metadata + 16;
    data[bfv + 4..bfv + 8].copy_from_slice(&4096u32.to_le_bytes());
    data[bfv + 8..bfv + 16].copy_from_slice(&0xffff_f000u64.to_le_bytes());
    data[bfv + 16..bfv + 24].copy_from_slice(&4096u64.to_le_bytes());
    data[bfv + 28..bfv + 32].copy_from_slice(&1u32.to_le_bytes());
    let hob = bfv + 32;
    data[hob + 8..hob + 16].copy_from_slice(&0x9000u64.to_le_bytes());
    data[hob + 16..hob + 24].copy_from_slice(&8192u64.to_le_bytes());
    data[hob + 24..hob + 28].copy_from_slice(&2u32.to_le_bytes());
    let temporary = hob + 32;
    data[temporary + 8..temporary + 16].copy_from_slice(&0x4000u64.to_le_bytes());
    data[temporary + 16..temporary + 24].copy_from_slice(&4096u64.to_le_bytes());
    data[temporary + 24..temporary + 28].copy_from_slice(&3u32.to_le_bytes());
    let footer = data.len() - 32 - 18;
    let locator = footer - 22;
    data[locator..locator + 4].copy_from_slice(&((PAGE - metadata) as u32).to_le_bytes());
    data[locator + 4..locator + 6].copy_from_slice(&22u16.to_le_bytes());
    data[locator + 6..footer].copy_from_slice(&OVMF_META_TDX_LIST.to_bytes());
    data[footer..footer + 2].copy_from_slice(&40u16.to_le_bytes());
    data[footer + 2..footer + 18].copy_from_slice(&OVMF_META_LIST.to_bytes());
    data
}

fn wrap(dir: &TestDir, firmware: &[u8], options: &[&str]) -> std::process::Output {
    let input = dir.0.join("input.fd");
    let output = dir.0.join("output.igvm");
    fs::write(&input, firmware).unwrap();
    Command::new(env!("CARGO_BIN_EXE_igvm-wrap"))
        .arg("--tdx")
        .args(options)
        .arg("-i")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap()
}

fn read_output(dir: &TestDir) -> IgvmFile {
    let bytes = fs::read(dir.0.join("output.igvm")).unwrap();
    IgvmFile::new_from_binary(&bytes, None).unwrap()
}

fn measure_page(hash: &mut Sha384, gpa: u64, bytes: &[u8], measured: bool) {
    let mut operation = [0u8; 128];
    operation[..12].copy_from_slice(b"MEM.PAGE.ADD");
    operation[16..24].copy_from_slice(&gpa.to_le_bytes());
    hash.update(operation);
    if measured {
        let mut page = [0u8; PAGE];
        page[..bytes.len()].copy_from_slice(bytes);
        for (index, chunk) in page.chunks_exact(256).enumerate() {
            let mut extend = [0u8; 128];
            extend[..9].copy_from_slice(b"MR.EXTEND");
            extend[16..24].copy_from_slice(&(gpa + (index * 256) as u64).to_le_bytes());
            hash.update(extend);
            hash.update(chunk);
        }
    }
}

#[test]
fn wraps_tdvf_in_metadata_order() {
    let dir = TestDir::new();
    let firmware = firmware();
    let result = wrap(&dir, &firmware, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let igvm = read_output(&dir);
    assert_eq!(igvm.platforms().len(), 1);
    let IgvmPlatformHeader::SupportedPlatform(platform) = &igvm.platforms()[0];
    assert_eq!(platform.platform_type, IgvmPlatformType::TDX);
    let mask = platform.compatibility_mask;
    assert_ne!(mask, virtfw_igvm_tools::builder::NATIVE_COMPAT);
    let mut area = None;
    let mut transcript = Sha384::new();
    let mut imports = Vec::new();
    for directive in igvm.directives() {
        match directive {
            IgvmDirectiveHeader::ParameterArea {
                number_of_bytes,
                parameter_area_index,
                ..
            } => {
                assert_eq!(*number_of_bytes, 8192);
                assert_eq!(*parameter_area_index, 0);
                assert!(area.replace(*parameter_area_index).is_none());
            }
            IgvmDirectiveHeader::PageData {
                gpa,
                compatibility_mask,
                flags,
                data_type,
                data,
            } => {
                assert_eq!(*compatibility_mask, mask);
                assert_eq!(*data_type, IgvmPageDataType::NORMAL);
                assert!(!flags.shared());
                imports.push(*gpa);
                measure_page(&mut transcript, *gpa, data, !flags.unmeasured());
            }
            IgvmDirectiveHeader::ParameterInsert(insert) => {
                assert_eq!(insert.compatibility_mask, mask);
                assert_eq!(Some(insert.parameter_area_index), area);
                assert_eq!(insert.gpa, 0x9000);
                imports.push(insert.gpa);
                measure_page(&mut transcript, insert.gpa, &[], false);
                measure_page(&mut transcript, insert.gpa + PAGE as u64, &[], false);
            }
            _ => {}
        }
    }
    assert!(area.is_some());
    assert_eq!(imports, [0xffff_f000, 0x9000, 0x4000]);
    let mut expected = Sha384::new();
    measure_page(&mut expected, 0xffff_f000, &firmware, true);
    measure_page(&mut expected, 0x9000, &[], false);
    measure_page(&mut expected, 0xa000, &[], false);
    measure_page(&mut expected, 0x4000, &[], false);
    assert_eq!(transcript.finalize(), expected.finalize());
}

fn dsdt(size: usize) -> Vec<u8> {
    let mut table = vec![0u8; size];
    table[..4].copy_from_slice(b"DSDT");
    table[4..8].copy_from_slice(&(size as u32).to_le_bytes());
    table[9] = 0u8.wrapping_sub(table.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)));
    table
}

fn ram() -> MemoryResource {
    MemoryResource {
        start: 0,
        length: 0x100_0000,
        kind: ResourceType::Unaccepted,
    }
}

fn prepare(
    file: &IgvmFile,
    resources: &[MemoryResource],
    tables: &[&[u8]],
) -> Result<virtfw_igvm_tools::tdvf_launch::TdvfLaunch, String> {
    let IgvmPlatformHeader::SupportedPlatform(platform) = &file.platforms()[0];
    prepare_tdvf_launch(file, platform.compatibility_mask, resources, tables)
}

fn launch(file: &IgvmFile) -> Result<virtfw_igvm_tools::tdvf_launch::TdvfLaunch, String> {
    prepare(file, &[ram()], &[&dsdt(36)])
}

fn with_directives(file: &IgvmFile, directives: Vec<IgvmDirectiveHeader>) -> IgvmFile {
    IgvmFile::new(
        IgvmRevision::V2 {
            arch: Arch::X64,
            page_size: 4096,
        },
        file.platforms().to_vec(),
        file.initializations().to_vec(),
        directives,
    )
    .unwrap()
}

#[test]
fn standard_igvm_launch_and_metadata_validation() {
    let dir = TestDir::new();
    let firmware = firmware();
    let result = wrap(&dir, &firmware, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let file = read_output(&dir);
    assert!(!file.directives().iter().any(|directive| matches!(
        directive,
        IgvmDirectiveHeader::SnpVpContext { .. }
            | IgvmDirectiveHeader::X64NativeVpContext { .. }
            | IgvmDirectiveHeader::X64VbsVpContext { .. }
            | IgvmDirectiveHeader::AArch64VbsVpContext { .. }
            | IgvmDirectiveHeader::AArch64CcaVpContext { .. }
    )));
    let prepared = launch(&file).unwrap();
    assert_eq!(prepared.initial_rcx, 0x9000);
    assert_eq!(
        prepared
            .pages
            .iter()
            .map(|page| (page.gpa, page.measured))
            .collect::<Vec<_>>(),
        [
            (0xffff_f000, true),
            (0x9000, false),
            (0xa000, false),
            (0x4000, false)
        ]
    );
    assert_eq!(prepared.pages[0].data, firmware);

    let original = file.directives().to_vec();
    let page_indices: Vec<_> = original
        .iter()
        .enumerate()
        .filter_map(|(index, directive)| {
            matches!(directive, IgvmDirectiveHeader::PageData { .. }).then_some(index)
        })
        .collect();
    assert_eq!(page_indices.len(), 2);
    let mut directives = original.clone();
    directives.swap(page_indices[0], page_indices[1]);
    assert!(launch(&with_directives(&file, directives)).is_err());
    let mut directives = original.clone();
    if let IgvmDirectiveHeader::PageData { flags, .. } = &mut directives[page_indices[1]] {
        *flags = flags.with_unmeasured(false);
    }
    assert!(launch(&with_directives(&file, directives)).is_err());
    let mut directives = original.clone();
    directives.remove(page_indices[1]);
    assert!(launch(&with_directives(&file, directives)).is_err());
    let mut directives = original.clone();
    for directive in &mut directives {
        if let IgvmDirectiveHeader::ParameterInsert(insert) = directive {
            insert.gpa = 0xb000;
        }
    }
    assert!(launch(&with_directives(&file, directives)).is_err());
    let mut directives = original;
    for directive in &mut directives {
        if let IgvmDirectiveHeader::ParameterArea {
            number_of_bytes, ..
        } = directive
        {
            *number_of_bytes = 4096;
        }
    }
    assert!(launch(&with_directives(&file, directives)).is_err());
}

#[test]
fn validates_runtime_resources_and_acpi() {
    let dir = TestDir::new();
    assert!(wrap(&dir, &firmware(), &[]).status.success());
    let file = read_output(&dir);
    let prepared = launch(&file).unwrap();
    let hob = &prepared.pages[1].data;
    assert_eq!(u16::from_le_bytes(hob[0..2].try_into().unwrap()), 1);
    let end = u64::from_le_bytes(hob[48..56].try_into().unwrap());
    let mut offset = 56;
    let mut resources = Vec::new();
    while 0x9000 + (offset as u64) < end {
        let kind = u16::from_le_bytes(hob[offset..offset + 2].try_into().unwrap());
        let length = u16::from_le_bytes(hob[offset + 2..offset + 4].try_into().unwrap()) as usize;
        if kind == 3 {
            resources.push((
                u32::from_le_bytes(hob[offset + 24..offset + 28].try_into().unwrap()),
                u64::from_le_bytes(hob[offset + 32..offset + 40].try_into().unwrap()),
                u64::from_le_bytes(hob[offset + 40..offset + 48].try_into().unwrap()),
            ));
        }
        offset += length;
    }
    assert_eq!(
        resources,
        [
            (7, 0, 0x4000),
            (0, 0x4000, 0x1000),
            (7, 0x5000, 0x4000),
            (0, 0x9000, 0x2000),
            (7, 0xb000, 0xff5000)
        ]
    );
    let table = dsdt(36);
    let mut invalid = ram();
    invalid.kind = ResourceType::Accepted;
    assert!(prepare(&file, &[invalid], &[&table]).is_err());
    invalid = MemoryResource {
        start: u64::MAX - 4095,
        length: 4096,
        kind: ResourceType::Unaccepted,
    };
    assert!(prepare(&file, &[invalid], &[&table]).is_err());
    assert!(prepare(&file, &[ram(), ram()], &[&table]).is_err());
    invalid = MemoryResource {
        start: 0,
        length: 0x9000,
        kind: ResourceType::Unaccepted,
    };
    assert!(prepare(&file, &[invalid], &[&table]).is_err());
    invalid = MemoryResource {
        start: 0x4000,
        length: 0x1000,
        kind: ResourceType::Reserved,
    };
    let split_ram = [
        MemoryResource {
            start: 0,
            length: 0x4000,
            ..ram()
        },
        invalid,
        MemoryResource {
            start: 0x5000,
            length: 0xffb000,
            ..ram()
        },
    ];
    assert!(prepare(&file, &split_ram, &[&table]).is_err());
    assert!(prepare(&file, &[ram()], &[&dsdt(8192)]).is_err());
    let mut bad = table.clone();
    bad[10] = 1;
    assert!(prepare(&file, &[ram()], &[&bad]).is_err());
    assert!(prepare(&file, &[ram()], &[]).is_err());
}

#[test]
fn rejects_invalid_embedded_metadata_and_parameter_writers() {
    let dir = TestDir::new();
    assert!(wrap(&dir, &firmware(), &[]).status.success());
    let file = read_output(&dir);
    let original = file.directives().to_vec();
    for mutation in 0..6 {
        let mut directives = original.clone();
        for directive in &mut directives {
            if let IgvmDirectiveHeader::PageData {
                gpa, data, flags, ..
            } = directive
            {
                if *gpa == 0xffff_f000 {
                    match mutation {
                        0 => data[PAGE - 32 - 16] ^= 1,
                        1 => data[72..76].copy_from_slice(&2u32.to_le_bytes()),
                        2 => data[68..72].copy_from_slice(&u32::MAX.to_le_bytes()),
                        3 => data[PAGE - 32 - 18 - 22..PAGE - 32 - 18 - 18]
                            .copy_from_slice(&(PAGE as u32 + 4096).to_le_bytes()),
                        4 => *flags = flags.with_unmeasured(true),
                        _ => {}
                    }
                } else if mutation == 5 {
                    data.resize(PAGE, 0);
                    data[0] = 1;
                }
            }
        }
        assert!(
            launch(&with_directives(&file, directives)).is_err(),
            "mutation {mutation}"
        );
    }
    let mut directives = original.clone();
    for directive in &mut directives {
        match directive {
            IgvmDirectiveHeader::ParameterArea {
                parameter_area_index,
                ..
            } => *parameter_area_index = 7,
            IgvmDirectiveHeader::ParameterInsert(insert) => insert.parameter_area_index = 7,
            _ => {}
        }
    }
    assert!(launch(&with_directives(&file, directives)).is_ok());
    let mut directives = original.clone();
    let mask = if let IgvmDirectiveHeader::ParameterInsert(insert) = directives
        .iter()
        .find(|directive| matches!(directive, IgvmDirectiveHeader::ParameterInsert(_)))
        .unwrap()
    {
        insert.compatibility_mask
    } else {
        unreachable!()
    };
    directives.push(IgvmDirectiveHeader::ParameterArea {
        parameter_area_index: 1,
        number_of_bytes: 4096,
        initial_data: vec![],
    });
    directives.push(IgvmDirectiveHeader::ParameterInsert(
        igvm_defs::IGVM_VHS_PARAMETER_INSERT {
            gpa: 0xc000,
            compatibility_mask: mask,
            parameter_area_index: 1,
        },
    ));
    assert!(launch(&with_directives(&file, directives)).is_err());
    let mut directives = original;
    let insert = directives
        .iter()
        .position(|directive| matches!(directive, IgvmDirectiveHeader::ParameterInsert(_)))
        .unwrap();
    directives.insert(
        insert,
        IgvmDirectiveHeader::MemoryMap(igvm_defs::IGVM_VHS_PARAMETER {
            parameter_area_index: 0,
            byte_offset: 0,
        }),
    );
    assert!(launch(&with_directives(&file, directives)).is_err());
}

#[test]
fn rejects_overlap_with_second_hob_page() {
    use virtfw_igvm_tools::builder::Builder;

    let mut builder = Builder::new();
    builder.add_tdvf(&firmware()).unwrap();
    builder.add_data_pages(0xa000, &[1]);
    assert!(builder.verify().unwrap_err().contains("overlap"));
}

#[test]
fn rejects_invalid_firmware_and_conflicting_options() {
    let dir = TestDir::new();
    let result = wrap(&dir, b"invalid", &[]);
    assert!(!result.status.success());
    assert!(!dir.0.join("output.igvm").exists());
    for option in ["--snp", "--native", "--real16", "--flat32"] {
        let result = wrap(&dir, &firmware(), &[option]);
        assert!(!result.status.success(), "{option}");
        assert!(!dir.0.join("output.igvm").exists());
    }
    let vars = dir.0.join("vars.fd");
    fs::write(&vars, [0u8; 4096]).unwrap();
    let result = wrap(&dir, &firmware(), &["--vars", vars.to_str().unwrap()]);
    assert!(!result.status.success());
    assert!(!dir.0.join("output.igvm").exists());
}

#[test]
fn default_native_wrap_preserves_firmware_aliases() {
    let dir = TestDir::new();
    let input = dir.0.join("native.fd");
    let output = dir.0.join("output.igvm");
    let firmware: Vec<u8> = (0..PAGE).map(|index| (index % 251) as u8).collect();
    fs::write(&input, &firmware).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_igvm-wrap"))
        .arg("-i")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let igvm = read_output(&dir);
    assert_eq!(igvm.platforms().len(), 1);
    let IgvmPlatformHeader::SupportedPlatform(platform) = &igvm.platforms()[0];
    assert_eq!(platform.platform_type, IgvmPlatformType::NATIVE);
    let pages: Vec<_> = igvm
        .directives()
        .iter()
        .filter_map(|directive| {
            if let IgvmDirectiveHeader::PageData { gpa, data, .. } = directive {
                Some((*gpa, data.as_slice()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        pages,
        [
            (0xff000, firmware.as_slice()),
            (0xffff_f000, firmware.as_slice())
        ]
    );
}

#[test]
#[ignore = "requires local TDX-OVMF.fd"]
fn wraps_real_firmware() {
    let firmware =
        fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../TDX-OVMF.fd")).unwrap();
    assert_eq!(Tdvf::parse(&firmware).unwrap().sections.len(), 6);
    let dir = TestDir::new();
    let result = wrap(&dir, &firmware, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let igvm = read_output(&dir);
    let prepared = launch(&igvm).unwrap();
    assert_eq!(prepared.initial_rcx, 0x809000);
    assert_eq!(prepared.pages.len(), 1050);
    assert_eq!(
        prepared.pages.iter().filter(|page| page.measured).count(),
        892
    );
    let mut total = 0;
    let mut measured = 0;
    let mut output_hash = Sha384::new();
    for directive in igvm.directives() {
        match directive {
            IgvmDirectiveHeader::PageData {
                gpa, flags, data, ..
            } => {
                total += 1;
                measured += usize::from(!flags.unmeasured());
                measure_page(&mut output_hash, *gpa, data, !flags.unmeasured());
            }
            IgvmDirectiveHeader::ParameterInsert(insert) => {
                total += 2;
                measure_page(&mut output_hash, insert.gpa, &[], false);
                measure_page(&mut output_hash, insert.gpa + PAGE as u64, &[], false);
            }
            _ => {}
        }
    }
    assert_eq!(total, 1050);
    assert_eq!(measured, 892);

    // Decode the sample's raw metadata independently of the production parser.
    let metadata = firmware
        .windows(4)
        .position(|bytes| bytes == b"TDVF")
        .unwrap();
    let count = u32::from_le_bytes(firmware[metadata + 12..metadata + 16].try_into().unwrap());
    assert_eq!(count, 6);
    let mut expected_hash = Sha384::new();
    for entry in firmware[metadata + 16..metadata + 16 + count as usize * 32].chunks_exact(32) {
        let raw_offset = u32::from_le_bytes(entry[0..4].try_into().unwrap()) as usize;
        let raw_size = u32::from_le_bytes(entry[4..8].try_into().unwrap()) as usize;
        let gpa = u64::from_le_bytes(entry[8..16].try_into().unwrap());
        let size = u64::from_le_bytes(entry[16..24].try_into().unwrap());
        let measured = u32::from_le_bytes(entry[28..32].try_into().unwrap()) == 1;
        let raw = &firmware[raw_offset..raw_offset + raw_size];
        for offset in (0..size as usize).step_by(PAGE) {
            let page = raw.get(offset..).unwrap_or_default();
            measure_page(
                &mut expected_hash,
                gpa + offset as u64,
                &page[..page.len().min(PAGE)],
                measured,
            );
        }
    }
    let expected_hash = expected_hash.finalize();
    println!("sample MRTD: {expected_hash:x}");
    assert_eq!(output_hash.finalize(), expected_hash);
}
