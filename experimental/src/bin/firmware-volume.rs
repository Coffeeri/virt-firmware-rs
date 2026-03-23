use clap::{CommandFactory, Parser};
use log::{error, info};
use std::process::ExitCode;

use virtfw_experimental::fwvol::{FirmwareItemContent, FirmwareItemHeader};

#[derive(Parser, Debug)]
#[command(version, author, name = "firmware-volume",
          about = "parse edk2 firmware images",
          long_about = None)]
struct Args {
    /// dump firmware structure
    #[arg(short, long)]
    dump: bool,

    /// list firmware volumes
    #[arg(short, long)]
    volumes: bool,

    /// files to inspect
    #[arg()]
    files: Vec<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

fn walk_print(level: usize, item: &FirmwareItemContent) {
    println!("{:indent$}{item}", "", indent = level * 2)
}

fn walk_volume(_level: usize, item: &FirmwareItemContent) {
    if let FirmwareItemHeader::Volume(_) = item.hdr {
        println!("{item}");
    }
}

fn parse_file(cfg: &Args, filename: &str) {
    info!("reading {filename}");
    let res = std::fs::read(filename);
    if let Err(e) = res {
        error!("read: {e}");
        return;
    }

    let blob = res.unwrap();
    if cfg.dump {
        FirmwareItemContent::new_image(&blob).walk(1, walk_print);
    }
    if cfg.volumes {
        FirmwareItemContent::new_image(&blob).walk(1, walk_volume);
    }
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), log::LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();

    for arg in &cfg.files {
        parse_file(&cfg, arg);
    }

    0.into()
}
