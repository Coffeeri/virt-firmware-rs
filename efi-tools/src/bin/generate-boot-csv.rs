//!
//! Generate BOOT.CVS for UKIs
//!

use clap::{CommandFactory, Parser};
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use virtfw_efi_tools::shimcsv::write_boot_csv;
use virtfw_libefi::arch::EfiArch;

#[derive(Parser, Debug)]
#[command(
    version,
    author,
    name = "generate-boot-csv",
    about = "create BOOT.CSV file for fallback.efi",
    long_about = r#"

Generate BOOT.CSV file for fallback.efi with entries for all UKIs
found in the $ESP/EFI/Linux.  Additional EFI binaries can be passed
via --add command line switch.

This utility intentionally does not try to autodetect mount points or
read EFI variables.  So it can safely be used in environments like the
anaconda installer (kickstart scripts) or image builders.  This
implies the utility can not autodetect where your ESP is, so you have
to use the --esp switch if your ESP is not mounted at /efi.

"#
)]
struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    loglevel: log::Level,

    /// set efi system partition location
    #[arg(long, value_name = "ESP")]
    esp: Option<String>,

    /// add efi binary
    #[arg(long, value_name = "EFI")]
    add: Vec<String>,

    /// override csv output location
    #[arg(long, value_name = "CSV")]
    csv: Option<String>,

    /// override kernel cmd line (default: /etc/kernel/cmdline)
    #[arg(long, value_name = "APPEND")]
    cmdline: Option<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

fn create_csv_line(esp_path: &Path, shim_name: &str, binary: &Path, cmdline: &str) -> String {
    let efi = binary
        .strip_prefix(esp_path)
        .unwrap()
        .display()
        .to_string()
        .replace("/", "\\");
    info!("efi: -> \\{efi}");
    let title = binary.file_name().unwrap().to_str().unwrap();
    let line = format!(
        "{},{},\\{} {},{}\n",
        shim_name,
        title,
        efi,
        cmdline.trim(),
        "Comment"
    );
    line
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    let levelfilter = args.loglevel.to_level_filter();
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();

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
    info!("cmdline: {}", cmdline.trim());

    let mut csv = String::new();
    let esp_path = std::path::Path::new(&esp);

    for app in args.add {
        if !app.starts_with(&esp) {
            error!("{app} is not on esp ({esp})");
            return ExitCode::from(1);
        }
        let binary = std::path::Path::new(&app);
        info!("add: {}", binary.display());
        let line = create_csv_line(esp_path, &shim_name, binary, "");
        csv.push_str(&line);
    }

    for uki in glob::glob(&uki_glob).expect("uki glob failed").flatten() {
        info!("uki: {}", uki.display());
        let line = create_csv_line(esp_path, &shim_name, &uki, &cmdline);
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
