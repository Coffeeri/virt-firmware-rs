use clap::Parser;
use log::{debug, error};
use std::process::{Command, ExitCode};

use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

use virtfw_efi_tools::cmdline::uefi_boot_menu::Args;
use virtfw_libefi::bootcfg;
use virtfw_libefi::efivar::boot::BootIndex;
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::varstore::sysfs;

fn main() -> ExitCode {
    let cfg = Args::parse();

    let levelfilter = if cfg.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .filter_module("virtfw_libefi", levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let bootcfg = bootcfg::BootConfig::new();
    let menulist = bootcfg.index_list();

    let mut titlelist = Vec::new();
    for i in &menulist {
        titlelist.push(bootcfg.index_title(i));
    }

    let theme = ColorfulTheme::default();
    let sel = Select::with_theme(&theme)
        .with_prompt("UEFI boot menu")
        .items(&titlelist)
        .interact_opt()
        .unwrap();

    let Some(s) = sel else {
        println!("Canceled.");
        return ExitCode::from(1);
    };

    let bi: &BootIndex = menulist[s];
    let bn = EfiVar::new_with_vec(
        ids::BOOT_NEXT.into(),
        EfiVarAttr::new_nv_bs_rt(),
        Vec::from(bi),
    );
    debug!("{bn:?}");
    println!("Setting BootNext to {bi}");
    if let Err(e) = sysfs::varstore_write(&bn) {
        error!("failed to write BootNext: {e}");
        return ExitCode::from(1);
    }

    if cfg.reboot {
        println!("Rebooting");
        let _ = Command::new("reboot")
            .spawn()
            .expect("failed to reboot")
            .wait();
    }

    ExitCode::from(0)
}
