use clap::Parser;
use log::{error, info};
use std::process::ExitCode;

use virtfw_experimental::pcrlock::Pcrlock;

#[derive(Parser, Debug)]
#[command(version, author, name = "pcrlock-parse",
          about = "parse pcrlock files",
          long_about = None)]
struct Args {
    /// files to inspect
    #[arg()]
    files: Vec<String>,
}

fn print_one(p: Pcrlock) {
    for r in p.records {
        println!("    pcr: {}", r.pcr);
        println!("    content: {}", r.content);
        for d in r.digests {
            println!("    digest: {}", d);
        }
        println!("    --");
    }
}

fn parse_one(filename: &str) -> Result<Pcrlock, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(filename)?;
    let pcrlock = serde_json::from_str(&json)?;
    Ok(pcrlock)
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), log::LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .init();

    for file in &cfg.files {
        match parse_one(file) {
            Ok(p) => {
                info!("{file}");
                print_one(p);
            }
            Err(e) => {
                error!("{file}: {e}");
            }
        }
    }

    0.into()
}
