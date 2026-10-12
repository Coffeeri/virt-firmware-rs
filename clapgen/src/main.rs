use clap::{Command, CommandFactory, Parser};
use log::error;
use std::process::ExitCode;

use virtfw_efi_tools::cmdline::check_sb_signature;
use virtfw_efi_tools::cmdline::enroll_sb_vars;
use virtfw_efi_tools::cmdline::generate_boot_csv;
use virtfw_efi_tools::cmdline::list_sb_vars;
use virtfw_efi_tools::cmdline::mini_bootcfg;
use virtfw_efi_tools::cmdline::uefi_boot_menu;

#[derive(Parser, Debug)]
#[command(name = "clapgen", about = "generate docs for virt-firmware-rs")]
pub struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    pub loglevel: log::Level,

    /// generate man page for command
    #[arg(long, value_name = "CMD")]
    manpage: Option<String>,
}

fn name_to_command(name: &str) -> Result<Command, Box<dyn std::error::Error>> {
    match name {
        "check-sb-signature" => Ok(check_sb_signature::Args::command()),
        "enroll-sb-vars" => Ok(enroll_sb_vars::Args::command()),
        "generate-boot-csv" => Ok(generate_boot_csv::Args::command()),
        "list-sb-vars" => Ok(list_sb_vars::Args::command()),
        "mini-bootcfg" => Ok(mini_bootcfg::Args::command()),
        "uefi-boot-menu" => Ok(uefi_boot_menu::Args::command()),
        _ => Err(format!("unknown command: {name}").into()),
    }
}

fn manpage(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = name_to_command(name)?;
    let man = clap_mangen::Man::new(cmd);
    man.render(&mut std::io::stdout())?;
    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    let levelfilter = args.loglevel.to_level_filter();
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();

    if let Some(name) = args.manpage {
        if let Err(e) = manpage(&name) {
            error!("{e}");
            return 1.into();
        }
    }

    0.into()
}
