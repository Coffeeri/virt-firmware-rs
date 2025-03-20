use log::{debug, error, info};
use std::env;
use std::process::ExitCode;
use std::str::FromStr;

use libefi::distro::efi_arch;

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optopt("l", "loglevel", "set loglevel", "LEVEL");

    opts.optopt("", "esp", "set efi system partition location", "ESP");

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

    let mut csv_path = shim_path.to_owned();
    csv_path.pop();
    csv_path.push(format!("BOOT{}.CSV", arch.to_uppercase()));
    info!("csv: {}", csv_path.display());

    let linux_glob = format!("{esp}/EFI/Linux/*.efi");
    debug!("find kernels: {linux_glob}");

    let mut csv = String::new();
    for linux in glob::glob(&linux_glob)
        .expect("linux glob failed")
        .flatten()
    {
        info!("linux: {}", linux.display());
        let title = linux.file_name().unwrap().to_str().unwrap();
        let line = format!(
            "{},{},{} ,{}\n",
            shim_name,
            title,
            linux.display(),
            "Comment"
        );
        csv.push_str(&line);
    }
    if csv.is_empty() {
        error!("no linux kernels found ({linux_glob})");
        return ExitCode::from(1);
    }
    print!("{}", csv);

    let mut data: Vec<u8> = Vec::new();
    data.push(0xff);
    data.push(0xfe);
    for c in csv.encode_utf16() {
        for b in c.to_le_bytes() {
            data.push(b);
        }
    }
    if let Err(e) = std::fs::write(&csv_path, data) {
        error!("write {}: {}", csv_path.display(), e);
        return ExitCode::from(1);
    }

    ExitCode::from(0)
}
