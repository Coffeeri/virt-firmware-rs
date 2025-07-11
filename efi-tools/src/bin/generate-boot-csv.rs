//!
//! Generate BOOT.CVS for UKIs
//!

use log::{debug, error, info};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use virtfw_efi_tools::shimcsv::write_boot_csv;
use virtfw_libefi::distro::efi_arch;

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optopt("l", "loglevel", "set loglevel", "LEVEL");

    opts.optopt("", "esp", "set efi system partition location", "ESP");
    opts.optopt("", "csv", "override csv output location", "CSV");

    let cfg_res = opts.parse(&args[1..]);
    let Ok(cfg) = cfg_res else {
        println!("{:?}", cfg_res.err());
        return None;
    };
    if cfg.opt_present("help") {
        print!("{}", opts.usage("generate-boot-csv"));
        return None;
    };

    Some(cfg)
}

fn loginit(cfg: &getopts::Matches) {
    let deflevel = stderrlog::LogLevelNum::Info;
    let loglevel = if let Some(s) = cfg.opt_str("loglevel") {
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
}

fn main() -> ExitCode {
    let Some(cfg) = parse_args() else {
        return ExitCode::from(1);
    };
    loginit(&cfg);

    let esp = cfg.opt_str("esp").unwrap_or("/efi".to_string());
    let arch = efi_arch();

    let shim_name = format!("shim{arch}.efi");
    let shim_glob = format!("{esp}/EFI/*/{shim_name}");
    debug!("find shim: {shim_glob}");

    let firstmatch = glob::glob(&shim_glob).expect("shim glob failed").next();
    let Some(Ok(shim_path)) = firstmatch else {
        error!("shim not found ({shim_glob})");
        return ExitCode::from(1);
    };
    info!("shim: {}", shim_path.display());

    let mut csv_path;
    if let Some(csv_opt) = cfg.opt_str("csv") {
        csv_path = PathBuf::from(csv_opt);
    } else {
        csv_path = shim_path.to_owned();
        csv_path.pop();
        csv_path.push(format!("BOOT{}.CSV", arch.to_uppercase()));
    }
    info!("csv: {}", csv_path.display());

    let uki_glob = format!("{esp}/EFI/Linux/*.efi");
    debug!("find kernels: {uki_glob}");

    let mut csv = String::new();
    for uki in glob::glob(&uki_glob).expect("uki glob failed").flatten() {
        info!("uki: {}", uki.display());
        let title = uki.file_name().unwrap().to_str().unwrap();
        let line = format!("{},{},{} ,{}\n", shim_name, title, uki.display(), "Comment");
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
