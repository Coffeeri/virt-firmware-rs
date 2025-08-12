use clap::Parser;
use log::info;
use std::process::ExitCode;

use igvm::{IgvmDirectiveHeader, IgvmFile};
use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;

#[derive(Parser, Debug)]
#[command(version, about = "igvm-inspect -- inspect igvm file content", long_about = None)]
struct Args {
    /// print ovmf metadata
    #[arg(long)]
    meta: bool,

    /// files to inspect
    #[arg()]
    files: Vec<String>,
}

fn resetvector(d: &IgvmDirectiveHeader) -> Option<&[u8]> {
    match d {
        IgvmDirectiveHeader::PageData { gpa, data, .. } => {
            if *gpa == 0xfffff000 {
                Some(data)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();

    for arg in &cfg.files {
        info!("inspecting {arg}");

        let blob = std::fs::read(arg).expect("file read error");
        let igvm = IgvmFile::new_from_binary(&blob, None).expect("file parse error");

        inspect_igvm(&igvm);

        if cfg.meta {
            for page in igvm.directives().iter().filter_map(resetvector) {
                if let Some(ovmfmeta) = OvmfMeta::new(page) {
                    ovmfmeta.print();
                }
            }
        }
    }

    ExitCode::from(0)
}
