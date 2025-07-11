use log::{debug, error};
use std::env;
use std::process::{Command, ExitCode};

use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

use virtfw_libefi::bootcfg;
use virtfw_libefi::efivar::boot::BootIndex;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;
use virtfw_libefi::varstore::sysfs;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optflag("d", "debug", "enable debug logging");
    opts.optflag("r", "reboot", "reboot after picking an entry");

    let cfg_res = opts.parse(&args[1..]);
    let Ok(cfg) = cfg_res else {
        println!("{:?}", cfg_res.err());
        return ExitCode::from(1);
    };
    if cfg.opt_present("help") {
        print!("{}", opts.usage("uefi boot menu"));
        return ExitCode::from(0);
    };

    let loglevel = if cfg.opt_present("debug") {
        stderrlog::LogLevelNum::Debug
    } else {
        stderrlog::LogLevelNum::Info
    };
    stderrlog::new()
        .module(module_path!())
        .module("libefi")
        .verbosity(loglevel)
        .init()
        .unwrap();

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
    let bn = EfiVar {
        name: "BootNext".to_string(),
        guid: guids::EfiGlobalVariable,
        attr: EfiVarAttr::new_nv_bs_rt(),
        data: Vec::from(bi),
    };
    debug!("{bn:?}");
    println!("Setting BootNext to {bi}");
    if let Err(e) = sysfs::varstore_write(&bn) {
        error!("failed to write BootNext: {e}");
        return ExitCode::from(1);
    }

    if cfg.opt_present("reboot") {
        println!("Rebooting");
        let _ = Command::new("reboot")
            .spawn()
            .expect("failed to reboot")
            .wait();
    }

    ExitCode::from(0)
}
