use clap::{Command, CommandFactory, Parser};
use clap_complete::aot::{generate, Shell};
use log::error;
use std::process::ExitCode;

use virtfw_efi_tools::cmdline::check_sb_signature;
use virtfw_efi_tools::cmdline::enroll_sb_vars;
use virtfw_efi_tools::cmdline::generate_boot_csv;
use virtfw_efi_tools::cmdline::list_sb_vars;
use virtfw_efi_tools::cmdline::mini_bootcfg;
use virtfw_efi_tools::cmdline::uefi_boot_menu;

use virtfw_igvm_tools::cmdline::igvm_elfis;
use virtfw_igvm_tools::cmdline::igvm_inspect;
use virtfw_igvm_tools::cmdline::igvm_update;
use virtfw_igvm_tools::cmdline::igvm_wrap;

#[derive(Parser, Debug)]
#[command(name = "clapgen", about = "generate docs for virt-firmware-rs")]
pub struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    pub loglevel: log::Level,

    /// generate man page for command
    #[arg(long, value_name = "CMD")]
    manpage: Option<String>,

    /// generate markdown file for command
    #[cfg(feature = "markdown")]
    #[arg(long, value_name = "CMD")]
    markdown: Option<String>,

    /// generate shell completions for command
    #[arg(long, value_name = "CMD")]
    complete: Option<String>,

    /// specify shell for completions
    #[arg(long, value_name = "SHELL", default_value = "bash")]
    shell: String,
}

fn name_to_command(name: &str) -> Result<Command, Box<dyn std::error::Error>> {
    match name {
        // efi tools
        "check-sb-signature" => Ok(check_sb_signature::Args::command()),
        "enroll-sb-vars" => Ok(enroll_sb_vars::Args::command()),
        "generate-boot-csv" => Ok(generate_boot_csv::Args::command()),
        "list-sb-vars" => Ok(list_sb_vars::Args::command()),
        "mini-bootcfg" => Ok(mini_bootcfg::Args::command()),
        "uefi-boot-menu" => Ok(uefi_boot_menu::Args::command()),

        // igvm tools
        "igvm-elfis" => Ok(igvm_elfis::Args::command()),
        "igvm-inspect" => Ok(igvm_inspect::Args::command()),
        "igvm-update" => Ok(igvm_update::Args::command()),
        "igvm-wrap" => Ok(igvm_wrap::Args::command()),

        _ => Err(format!("unknown command: {name}").into()),
    }
}

fn render_manpage(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = name_to_command(name)?;
    let man = clap_mangen::Man::new(cmd);
    man.render(&mut std::io::stdout())?;
    Ok(())
}

#[cfg(feature = "markdown")]
fn render_markdown(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = name_to_command(name)?;
    let md = clap_markdown::help_markdown_command(&cmd);
    print!("{md}");
    Ok(())
}

fn complete(name: &str, shell: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = name_to_command(name)?;
    let gen = Shell::from_shell_path(shell).ok_or("unknown shell")?;
    generate(gen, &mut cmd, name, &mut std::io::stdout());
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
        if let Err(e) = render_manpage(&name) {
            error!("{e}");
            return 1.into();
        }
    }

    #[cfg(feature = "markdown")]
    if let Some(name) = args.markdown {
        if let Err(e) = render_markdown(&name) {
            error!("{e}");
            return 1.into();
        }
    }

    if let Some(name) = args.complete {
        if let Err(e) = complete(&name, &args.shell) {
            error!("{e}");
            return 1.into();
        }
    }

    0.into()
}
