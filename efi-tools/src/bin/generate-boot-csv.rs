//!
//! Generate BOOT.CVS for UKIs
//!

use clap::Parser;
use log::{debug, error, info};
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use virtfw_efi_tools::shimcsv::write_boot_csv;
use virtfw_libefi::arch::EfiArch;

#[derive(Parser, Debug)]
#[command(version, name = "generate-boot-csv",
          about = "create BOOT.CSV file for fallback.efi",
          long_about = None)]
struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL")]
    loglevel: Option<String>,

    /// set efi system partition location
    #[arg(long, value_name = "ESP")]
    esp: Option<String>,

    /// override csv output location
    #[arg(long, value_name = "CSV")]
    csv: Option<String>,

    /// override kernel cmd line (default: /etc/kernel/cmdline)
    #[arg(long, value_name = "APPEND")]
    cmdline: Option<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let deflevel = stderrlog::LogLevelNum::Info;
    let loglevel = if let Some(s) = args.loglevel {
        if let Ok(l) = log::Level::from_str(&s) {
            stderrlog::LogLevelNum::from(l)
        } else {
            deflevel
        }
    } else {
        deflevel
    };
    stderrlog::new()
        .module(module_path!())
        .verbosity(loglevel)
        .init()
        .unwrap();

    let esp = args.esp.unwrap_or("/efi".to_string());
    let arch = EfiArch::native();

    let shim_name = format!("shim{}.efi", arch.name());
    let shim_glob = format!("{esp}/EFI/*/{shim_name}");
    debug!("find shim: {shim_glob}");

    let firstmatch = glob::glob(&shim_glob).expect("shim glob failed").next();
    let Some(Ok(shim_path)) = firstmatch else {
        error!("shim not found ({shim_glob})");
        return ExitCode::from(1);
    };
    info!("shim: {}", shim_path.display());

    let mut csv_path;
    if let Some(csv_opt) = args.csv {
        csv_path = PathBuf::from(csv_opt);
    } else {
        csv_path = shim_path.to_owned();
        csv_path.pop();
        csv_path.push(format!("BOOT{}.CSV", arch.name_uc()));
    }
    info!("csv: {}", csv_path.display());

    let uki_glob = format!("{esp}/EFI/Linux/*.efi");
    debug!("find kernels: {uki_glob}");

    let cmdline = match args.cmdline {
        Some(c) => c,
        None => std::fs::read_to_string("/etc/kernel/cmdline").unwrap_or("".into()),
    };
    info!("cmdline: {}", cmdline);

    let mut csv = String::new();
    let esp_path = std::path::Path::new(&esp);
    for uki in glob::glob(&uki_glob).expect("uki glob failed").flatten() {
        info!("uki: {}", uki.display());
        let efi = uki
            .strip_prefix(esp_path)
            .unwrap()
            .display()
            .to_string()
            .replace("/", "\\");
        info!("efi: -> \\{efi}");
        let title = uki.file_name().unwrap().to_str().unwrap();
        let line = format!(
            "{},{},\\{} {},{}\n",
            shim_name, title, efi, cmdline, "Comment"
        );
        csv.push_str(&line);
    }
    if csv.is_empty() {
        error!("no uki images found ({uki_glob})");
        return ExitCode::from(1);
    }
    print!("{csv}");

    if let Err(e) = write_boot_csv(&csv_path, &csv) {
        error!("write {}: {}", csv_path.display(), e);
        return ExitCode::from(1);
    }

    ExitCode::from(0)
}
