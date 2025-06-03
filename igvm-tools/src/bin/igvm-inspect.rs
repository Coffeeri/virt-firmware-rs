use log::info;
use std::env;

use igvm::{IgvmDirectiveHeader, IgvmFile, IgvmInitializationHeader, IgvmPlatformHeader};
use igvm_defs::{IgvmPageDataFlags, IgvmPageDataType, IgvmPlatformType, PAGE_SIZE_4K};

fn format_compat(igvm: &IgvmFile, compat: u32) -> String {
    let platforms: Vec<IgvmPlatformType> = igvm
        .platforms()
        .iter()
        .filter_map(|p| {
            let IgvmPlatformHeader::SupportedPlatform(sp) = p;
            if sp.compatibility_mask & compat != 0 {
                Some(sp.platform_type)
            } else {
                None
            }
        })
        .collect();

    format!("compat 0x{:x} {:?}", compat, platforms)
}

fn print_pagedata(igvm: &IgvmFile) {
    let mut next_gpa: Option<u64> = None;
    let mut prev_type: Option<IgvmPageDataType> = None;
    let mut prev_flags: Option<IgvmPageDataFlags> = None;
    let mut prev_compat: Option<u32> = None;
    let mut page_count: u64 = 0;

    for d in igvm
        .directives()
        .iter()
        .filter(|x| matches! {x, IgvmDirectiveHeader::PageData { .. }})
    {
        if let IgvmDirectiveHeader::PageData {
            gpa,
            flags,
            data_type,
            compatibility_mask,
            ..
        } = d
        {
            if next_gpa == Some(*gpa)
                && prev_type == Some(*data_type)
                && prev_flags == Some(*flags)
                && prev_compat == Some(*compatibility_mask)
            {
                next_gpa = Some(*gpa + PAGE_SIZE_4K);
                page_count += 1;
                continue;
            }
            if page_count > 0 {
                println!(
                    "PageData: end 0x{:x} ({} more pages)",
                    next_gpa.unwrap() - 1,
                    page_count
                );
                page_count = 0;
            }
            println!(
                "PageData: gpa 0x{:x}, type {:?}, flags {}{}{}, {}",
                gpa,
                data_type,
                if flags.is_2mb_page() { "2m" } else { "4k" },
                if flags.unmeasured() {
                    ",unmeasured"
                } else {
                    ""
                },
                if flags.shared() { ",shared" } else { "" },
                format_compat(igvm, *compatibility_mask)
            );
            next_gpa = Some(*gpa + PAGE_SIZE_4K);
            prev_type = Some(*data_type);
            prev_flags = Some(*flags);
            prev_compat = Some(*compatibility_mask);
        }
    }
    if page_count > 0 {
        println!(
            "PageData: end 0x{:x} ({} more pages)",
            next_gpa.unwrap() - 1,
            page_count
        );
    }
}

fn main() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    for arg in args {
        info!("{}", arg);

        let blob = std::fs::read(arg).expect("file read error");
        let igvm = IgvmFile::new_from_binary(&blob, None).expect("file parse error");

        for p in igvm.platforms() {
            let IgvmPlatformHeader::SupportedPlatform(sp) = p;
            println!(
                "platform: {:?}, compat 0x{:x}",
                sp.platform_type, sp.compatibility_mask
            );
        }

        for i in igvm.initializations() {
            match i {
                IgvmInitializationHeader::GuestPolicy {
                    policy,
                    compatibility_mask,
                } => {
                    println!(
                        "initialization: policy 0x{:x}, {}",
                        policy,
                        format_compat(&igvm, *compatibility_mask)
                    );
                }
                _ => {
                    println!("initialization: {:?}", i);
                }
            }
        }

        for d in igvm
            .directives()
            .iter()
            .filter(|x| !matches! {x, IgvmDirectiveHeader::PageData { .. }})
        {
            match d {
                IgvmDirectiveHeader::SnpVpContext {
                    compatibility_mask, ..
                } => {
                    println!(
                        "SnpVpContext: {}",
                        format_compat(&igvm, *compatibility_mask)
                    );
                }
                IgvmDirectiveHeader::X64NativeVpContext {
                    compatibility_mask, ..
                } => {
                    println!(
                        "X64NativeVpContext: {}",
                        format_compat(&igvm, *compatibility_mask)
                    );
                }

                IgvmDirectiveHeader::RequiredMemory {
                    gpa,
                    compatibility_mask,
                    number_of_bytes,
                    ..
                } => {
                    println!(
                        "RequiredMemory: gpa 0x{:x}, bytes 0x{:X}, {}",
                        gpa,
                        number_of_bytes,
                        format_compat(&igvm, *compatibility_mask)
                    );
                }

                IgvmDirectiveHeader::ParameterArea {
                    number_of_bytes,
                    parameter_area_index,
                    ..
                } => {
                    println!(
                        "ParameterArea: idx {}, bytes 0x{:x}",
                        parameter_area_index, number_of_bytes
                    );
                }

                IgvmDirectiveHeader::EnvironmentInfo(p) => {
                    println!(
                        "Parameter/EnvironmentInfo: idx {}, offset 0x{:x}",
                        p.parameter_area_index, p.byte_offset
                    );
                }
                IgvmDirectiveHeader::MemoryMap(p) => {
                    println!(
                        "Parameter/MemoryMap: idx {}, offset 0x{:x}",
                        p.parameter_area_index, p.byte_offset
                    );
                }
                IgvmDirectiveHeader::VpCount(p) => {
                    println!(
                        "Parameter/VpCount: idx {}, offset 0x{:x}",
                        p.parameter_area_index, p.byte_offset
                    );
                }

                IgvmDirectiveHeader::ParameterInsert(p) => {
                    println!(
                        "ParameterInsert: gpa 0x{:x}, idx {}, {}",
                        p.gpa,
                        p.parameter_area_index,
                        format_compat(&igvm, p.compatibility_mask)
                    );
                }

                _ => {
                    println!("directive: {}", d);
                }
            }
        }

        print_pagedata(&igvm);
    }
}
