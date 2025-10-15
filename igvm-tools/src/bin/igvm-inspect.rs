use clap::Parser;
use log::info;
use std::process::ExitCode;

use igvm::IgvmFile;
use virtfw_igvm_tools::inspect::{find_resetvector, inspect_igvm};
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;

#[derive(Parser, Debug)]
#[command(version, name = "igvm-inspect",  about = "inspect igvm file content", long_about = None)]
struct Args {
    /// print ovmf metadata
    #[arg(long)]
    meta: bool,

    /// files to inspect
    #[arg()]
    files: Vec<String>,
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
            for page in igvm.directives().iter().filter_map(find_resetvector) {
                if let Some(ovmfmeta) = OvmfMeta::new(page) {
                    ovmfmeta.print();
                }
            }
        }
    }

    ExitCode::from(0)
}
