use clap::Parser;
use log::{error, info};
use std::process::ExitCode;

use igvm::IgvmFile;
use virtfw_igvm_tools::cmdline::igvm_inspect::Args;
use virtfw_igvm_tools::inspect::{find_resetvector, inspect_igvm};
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;

fn print_one(filename: &str, ovmfmeta: bool) -> Result<(), Box<dyn std::error::Error>> {
    info!("inspecting {filename}");

    let blob = std::fs::read(filename)?;
    let igvm = IgvmFile::new_from_binary(&blob, None)?;

    inspect_igvm(&igvm);

    if ovmfmeta {
        for page in igvm.directives().iter().filter_map(find_resetvector) {
            if let Some(ovmfmeta) = OvmfMeta::new(page) {
                ovmfmeta.print();
            }
        }
    };

    Ok(())
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), log::LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .init();

    for arg in &cfg.files {
        if let Err(e) = print_one(arg, cfg.meta) {
            error!("{arg}: {e}");
        }
    }

    ExitCode::from(0)
}
