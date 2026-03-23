use clap::{CommandFactory, Parser};
use log::{error, info};
use std::process::ExitCode;

use virtfw_experimental::qemumeta::{FirmwareMapping, QemuFirmware};

#[derive(Parser, Debug)]
#[command(version, author, name = "qemu-fw-parse",
          about = "parse (and verify) and print qemu firmware description files",
          long_about = None)]
struct Args {
    /// print parsed file content
    #[arg(short, long)]
    print: bool,

    /// verify files (stop with exit code 1 on errors)
    #[arg(long)]
    verify: bool,

    /// files to inspect
    #[arg()]
    files: Vec<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

fn print_mapping(map: &FirmwareMapping) {
    match map {
        FirmwareMapping::Flash(f) => {
            println!("    map/flash");
            if let Some(m) = f.mode.as_ref() {
                println!("        mode    : {:?}", m);
            }
            println!("        code    : {:?}", f.executable);
            if let Some(v) = f.nvram_template.as_ref() {
                println!("        vars    : {:?}", v);
            }
        }
        FirmwareMapping::Igvm(i) => {
            println!("    map/igvm    : {:?}", i.filename);
        }
        FirmwareMapping::Kernel(k) => {
            println!("    map/kernel  : {:?}", k.filename);
        }
        FirmwareMapping::Memory(m) => {
            println!("    map/memory");
            println!("        code    : {:?}", m.filename);
            if let Some(v) = m.uefi_vars.as_ref() {
                println!("        vars    : {:?}", v.template);
            }
        }
    }
}

fn print_one(meta: QemuFirmware) {
    println!("    description : {:?}", meta.description);
    println!("    interface   : {:?}", meta.interface_types);
    print_mapping(&meta.mapping);

    println!("    targets");
    for t in meta.targets {
        println!("        target  : {:?}", t);
    }

    println!("    features    : {:?}", meta.features);
    println!("    tags        : {:?}", meta.tags);
    println!();
}

fn parse_one(filename: &str) -> Result<QemuFirmware, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(filename)?;
    let meta: QemuFirmware = serde_json::from_str(&json)?;
    Ok(meta)
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), log::LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .init();

    for arg in &cfg.files {
        match parse_one(arg) {
            Ok(meta) => {
                if cfg.print {
                    info!("print {arg}");
                    print_one(meta);
                } else {
                    info!("{arg}: OK");
                }
            }
            Err(e) => {
                error!("{arg}: {e}");
                if cfg.verify {
                    return 1.into();
                }
            }
        }
    }

    0.into()
}
