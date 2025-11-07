use clap::{CommandFactory, Parser};
use log::{error, info};
use std::process::ExitCode;

use igvm::IgvmFile;
use virtfw_igvm_tools::inspect::{find_resetvector, inspect_igvm};
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-inspect",
          about = "inspect igvm file content",
          long_about = None)]
struct Args {
    /// print ovmf metadata
    #[arg(long)]
    meta: bool,

    /// files to inspect
    #[arg()]
    files: Vec<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

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

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();

    for arg in &cfg.files {
        if let Err(e) = print_one(arg, cfg.meta) {
            error!("{arg}: {e}");
        }
    }

    ExitCode::from(0)
}
