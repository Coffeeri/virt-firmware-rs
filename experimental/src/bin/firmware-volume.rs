use clap::{CommandFactory, Parser};
use log::{error, info};
use std::process::ExitCode;

use virtfw_experimental::fwvol::FirmwareItemContent;

#[derive(Parser, Debug)]
#[command(version, author, name = "firmware-volume",
          about = "parse edk2 firmware images",
          long_about = None)]
struct Args {
    /// files to inspect
    #[arg()]
    files: Vec<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

fn walk_print(level: usize, item: &FirmwareItemContent) {
    println!("{:indent$}{item}", "", indent = level * 2)
}

fn parse_file(filename: &str) {
    info!("reading {filename}");
    let res = std::fs::read(filename);
    if let Err(e) = res {
        error!("read: {e}");
        return;
    }

    let blob = res.unwrap();
    FirmwareItemContent::new_image(&blob).walk(1, walk_print);
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
        .verbosity(stderrlog::LogLevelNum::Debug)
        .init()
        .unwrap();

    for arg in &cfg.files {
        parse_file(arg);
    }

    0.into()
}
