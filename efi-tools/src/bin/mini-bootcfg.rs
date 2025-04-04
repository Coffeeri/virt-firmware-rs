use log::{debug, error, info, warn};
use std::env;
use std::io;
use std::io::Cursor;
use std::process::ExitCode;
use std::str::FromStr;

use virtfw_libefi::bootcfg::BootConfig;
use virtfw_libefi::distro::LinuxOsInfo;
use virtfw_libefi::efifile::EfiFile;
use virtfw_libefi::efivar::boot;
use virtfw_libefi::efivar::devpath;
use virtfw_libefi::efivar::io::ReadEfiExt;
use virtfw_libefi::efivar::io::WriteEfiExt;
use virtfw_libefi::efivar::types::EfiVar;
use virtfw_libefi::efivar::types::EfiVarAttr;
use virtfw_libefi::guids;
use virtfw_libefi::nocasestr::NoCaseString;
use virtfw_libefi::varstore::sysfs;

use virtfw_efi_tools::shimcsv::write_boot_csv;

fn create_file_boot_entry(
    bootcfg: &BootConfig,
    shim_opt: Option<&EfiFile>,
    uki: &EfiFile,
    title: Option<String>,
    cmdline: Option<String>,
) -> Option<boot::BootEntry> {
    let devpath;
    let optdata;

    if let Some(shim) = shim_opt {
        if shim.devpathnode_hd() != uki.devpathnode_hd() {
            error!("shim and uki are on different partitions");
            return None;
        }
        devpath = shim.devpath();
        let Some(efipath) = uki.efi_name() else {
            error!("failed to format efi filename");
            return None;
        };
        optdata = match cmdline {
            Some(c) => Some(format!("{} {}", efipath, c)),
            None => Some(efipath.0),
        }
    } else {
        if bootcfg.sb {
            error!("shim not found and secure boot enabled");
            return None;
        }
        devpath = uki.devpath();
        optdata = cmdline.map(|c| c.to_string())
    };

    let t = match title {
        Some(t) => t,
        None => uki.basename().unwrap(),
    };

    Some(boot::BootEntry::new_boot(&t, devpath.unwrap(), optdata))
}

fn create_uri_boot_entry(
    uri: &str,
    title: Option<String>,
    cmdline: Option<String>,
) -> Option<boot::BootEntry> {
    let mut devpath = devpath::DevPath::new();
    devpath.push(devpath::DevPathNode::MsgUri(devpath::DevPathMsgUri {
        uri: Some(uri.to_string()),
    }));
    let optdata = cmdline.map(|c| c.to_string());
    let t = match title {
        Some(t) => t,
        None => uri.split('/').last().unwrap().to_string(),
    };
    Some(boot::BootEntry::new_boot(&t, devpath, optdata))
}

fn find_boot_entry_for_file<'cfg>(
    bootcfg: &'cfg BootConfig,
    shim_opt: Option<&EfiFile>,
    uki: &EfiFile,
) -> Option<&'cfg boot::BootIndex> {
    for (i, e) in &bootcfg.entries {
        if let Some(dp) = uki.devpath() {
            if e.devpath == dp {
                // entry device path matches uki
                return Some(i);
            }
        }
        if let Some(shim) = shim_opt {
            if let Some(dp) = shim.devpath() {
                if e.devpath == dp {
                    // entry device path matches shim, and ...
                    if let boot::BootEntryOptData::String { string: cmdline } = &e.optdata {
                        let mut iter = cmdline.split_ascii_whitespace();
                        let binary = NoCaseString::new(iter.next().unwrap());
                        if let Some(uki_path) = uki.efi_name() {
                            if uki_path == binary {
                                // ... entry optdata matches uki path
                                return Some(i);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn write_variable(var: &EfiVar, dryrun: bool) -> io::Result<()> {
    if dryrun {
        info!("skip update efivar: {} (dry-run)", var.name);
    } else {
        info!("update efivar: {}", var.name);
        sysfs::varstore_write(var)?;
    }
    Ok(())
}

fn delete_variable(name: &str, dryrun: bool) -> io::Result<()> {
    if dryrun {
        info!("skip delete efivar: {} (dry-run)", name);
    } else {
        info!("delete efivar: {}", name);
        sysfs::varstore_delete(name, &guids::EfiGlobalVariable)?;
    }
    Ok(())
}

fn write_bootentry(
    index: &boot::BootIndex,
    entry: boot::BootEntry,
    dryrun: bool,
) -> io::Result<()> {
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry)?;

    let be = EfiVar {
        name: format!("Boot{}", index),
        guid: guids::EfiGlobalVariable,
        attr: EfiVarAttr::new_nv_bs_rt(),
        data: wr.into_inner(),
    };

    info!(
        "set efivar: {} = \"{}\" {} {}",
        be.name, entry.title, entry.devpath, entry.optdata
    );
    write_variable(&be, dryrun)?;
    Ok(())
}

fn write_bootnext(index: &boot::BootIndex, dryrun: bool) -> io::Result<()> {
    let bn = EfiVar {
        name: "BootNext".to_string(),
        guid: guids::EfiGlobalVariable,
        attr: EfiVarAttr::new_nv_bs_rt(),
        data: Vec::from(index),
    };

    debug!("set efivar: {} = {}", bn.name, index);
    write_variable(&bn, dryrun)?;
    Ok(())
}

fn update_bootorder(
    new_order: &boot::BootOrder,
    old_order: &boot::BootOrder,
    dryrun: bool,
) -> io::Result<()> {
    let bo = EfiVar {
        name: "BootOrder".to_string(),
        guid: guids::EfiGlobalVariable,
        attr: EfiVarAttr::new_nv_bs_rt(),
        data: Vec::from(new_order),
    };

    debug!("old efivar: {} = {}", bo.name, old_order);
    debug!("set efivar: {} = {}", bo.name, new_order);
    if new_order.is_empty() {
        delete_variable("BootOrder", dryrun)?;
    } else {
        write_variable(&bo, dryrun)?;
    }
    Ok(())
}

fn add_bootindex(
    bootcfg: &BootConfig,
    index: &boot::BootIndex,
    cfg: &getopts::Matches,
) -> io::Result<()> {
    let dryrun = cfg.opt_present("dry-run");
    if cfg.opt_present("once") || cfg.opt_present("boot-next") {
        write_bootnext(index, dryrun)?;
    }
    if let Some(posstr) = cfg.opt_str("boot-order") {
        let old_order = if let Some(o) = &bootcfg.order {
            o
        } else {
            &boot::BootOrder::empty()
        };

        let pos = if posstr == "last" {
            old_order.len()
        } else {
            let Ok(p) = posstr.parse::<usize>() else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to parse pos: {}", posstr),
                ));
            };
            if p > old_order.len() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("pos is too big: {} > {}", p, old_order.len()),
                ));
            }
            p
        };

        let new_order = old_order.insert(pos, index);
        update_bootorder(&new_order, old_order, dryrun)?;
    }
    Ok(())
}

fn remove_bootentry(bootcfg: &BootConfig, index: &boot::BootIndex, dryrun: bool) -> io::Result<()> {
    let name = format!("Boot{}", index);
    delete_variable(&name, dryrun)?;

    if let Some(next) = bootcfg.next {
        if next == *index {
            delete_variable("BootNext", dryrun)?;
        }
    }

    if let Some(old_order) = &bootcfg.order {
        if let Some(pos) = old_order.position(index) {
            let new_order = old_order.remove(pos);
            update_bootorder(&new_order, old_order, dryrun)?;
        }
    }

    Ok(())
}

fn boot_successful(bootcfg: &BootConfig, dryrun: bool) -> io::Result<()> {
    let Some(current) = bootcfg.current else {
        warn!("BootCurrent not set");
        return Ok(());
    };

    let old_order = if let Some(o) = &bootcfg.order {
        o
    } else {
        &boot::BootOrder::empty()
    };

    if let Some(pos) = old_order.position(&current) {
        info!(
            "BootCurrent ({}) already in BootOrder (position {})",
            current, pos
        );
        return Ok(());
    }

    let new_order = old_order.insert(0, &current);
    update_bootorder(&new_order, old_order, dryrun)?;
    Ok(())
}

fn update_csv(bootcfg: &BootConfig, shim_opt: Option<&EfiFile>) -> io::Result<()> {
    let Some(order) = &bootcfg.order else {
        warn!("BootOrder not set");
        return Ok(());
    };
    let Some(shim) = shim_opt else {
        warn!("shim.efi not found");
        return Ok(());
    };
    let Some(devpath) = shim.devpath() else {
        warn!("no efi device path for shim.efi");
        return Ok(());
    };
    let Some(basename) = shim.basename() else {
        warn!("no basename for shim.efi");
        return Ok(());
    };

    let mut csvname = shim.file.clone();
    csvname.pop();
    csvname.push(
        basename
            .to_uppercase()
            .replace("SHIM", "BOOT")
            .replace("EFI", "CSV"),
    );
    info!("updating {}", csvname.display());

    let mut csvdata = String::new();
    for index in &order.0 {
        let Some(entry) = bootcfg.entries.get(index) else {
            continue;
        };
        if devpath != entry.devpath {
            continue;
        };

        let args = match &entry.optdata {
            boot::BootEntryOptData::String { string } => string,
            _ => "",
        };

        let line = format!("{},{},{} ,Comment", basename, entry.title, args);
        debug!("{}", line);
        csvdata.push_str(&line);
        csvdata.push('\n');
    }

    write_boot_csv(&csvname, &csvdata)?;
    Ok(())
}

fn show_bootcfg(bootcfg: &BootConfig, verbose: bool) {
    for i in bootcfg.index_list() {
        println!("{}", bootcfg.index_title(i));
        if verbose {
            if let Some(e) = bootcfg.entries.get(i) {
                println!("      devpath: {}", e.devpath);
                match &e.optdata {
                    boot::BootEntryOptData::String { string } => {
                        println!("      optdata/string: \"{}\"", string)
                    }
                    boot::BootEntryOptData::Guid { guid } => {
                        println!("      optdata/guid: {}", guid)
                    }
                    boot::BootEntryOptData::Data { bytes } => {
                        println!("      optdata/bytes: {:?}", bytes)
                    }
                    boot::BootEntryOptData::None => {}
                }
            }
            println!();
        }
    }
}

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optopt("l", "loglevel", "set loglevel", "LEVEL");

    // show status
    opts.optflag("s", "show", "print boot configuration");
    opts.optflag("v", "verbose", "more verbose output");

    // uki entry updates
    opts.optopt("", "add-uki", "add boot entry for UKI image FILE", "FILE");
    opts.optopt(
        "",
        "remove-uki",
        "remove boot entry for UKI image FILE",
        "FILE",
    );
    opts.optopt("", "cmdline", "override cmd line for boot entry", "CMDLINE");

    opts.optflag("", "boot-ok", "boot is successful.");
    opts.optflag("", "boot-successful", "boot is successful.");
    opts.optflag("", "update-csv", "update BOOT.CSV");

    // other entry updates
    opts.optopt("", "add-uri", "add boot entry to netboot URI", "URI");
    opts.optopt("", "remove-entry", "remove boot entry NNNN", "NNNN");

    // boot entry update options
    opts.optflag("", "once", "boot added entry once (via BootNext)");
    opts.optflag("", "boot-next", "boot added entry once (via BootNext)");
    opts.optopt(
        "",
        "boot-order",
        "place added entry at POS to BootOrder",
        "POS",
    );
    opts.optflag("", "dry-run", "do not actually update the configuration");
    opts.optopt("", "title", "set title for boot entry", "TITLE");

    // system info
    opts.optflag("", "print-loader", "print boot loader name");

    let cfg_res = opts.parse(&args[1..]);
    let Ok(cfg) = cfg_res else {
        println!("{:?}", cfg_res.err());
        return None;
    };
    if cfg.opt_present("help") {
        print!(
            "{}",
            opts.usage("bootcfg - show and manage uefi boot entries")
        );
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
        .module("libefi")
        .verbosity(loglevel)
        .init()
        .unwrap();
}

fn main() -> ExitCode {
    let Some(cfg) = parse_args() else {
        return ExitCode::from(1);
    };
    loginit(&cfg);
    let dryrun = cfg.opt_present("dry-run");
    let verbose = cfg.opt_present("verbose");

    let osinfo = LinuxOsInfo::new();
    let bootcfg = BootConfig::new();

    let shim = osinfo.shim_path().unwrap();
    let efi_shim = EfiFile::new(&shim);

    if cfg.opt_present("show") {
        show_bootcfg(&bootcfg, verbose);
        return ExitCode::from(0);
    }

    if let Some(uki) = cfg.opt_str("add-uki") {
        let efi_uki = match EfiFile::new(&uki) {
            Err(e) => {
                error!("{}: {}", uki, e);
                return ExitCode::from(1);
            }
            Ok(u) => u,
        };

        let idx_opt = find_boot_entry_for_file(&bootcfg, efi_shim.as_ref().ok(), &efi_uki);
        let index = match idx_opt {
            Some(idx) => {
                info!("entry exists for {} (Boot{})", uki, idx);
                idx
            }
            None => &bootcfg.index_unused(),
        };
        let entry_opt = create_file_boot_entry(
            &bootcfg,
            efi_shim.as_ref().ok(),
            &efi_uki,
            cfg.opt_str("title"),
            cfg.opt_str("cmdline"),
        );
        let Some(entry) = entry_opt else {
            error!("failed to create boot entry for {}", uki);
            return ExitCode::from(1);
        };
        if let Err(e) = write_bootentry(index, entry, dryrun) {
            error!("write boot entry: {}", e);
            return ExitCode::from(1);
        };
        if let Err(e) = add_bootindex(&bootcfg, index, &cfg) {
            error!("add boot index: {}", e);
            return ExitCode::from(1);
        };
        return ExitCode::from(0);
    }

    if let Some(uki) = cfg.opt_str("remove-uki") {
        let efi_uki = match EfiFile::new(&uki) {
            Err(e) => {
                error!("{}: {}", uki, e);
                return ExitCode::from(1);
            }
            Ok(u) => u,
        };

        let idx_opt = find_boot_entry_for_file(&bootcfg, efi_shim.as_ref().ok(), &efi_uki);
        let Some(idx) = idx_opt else {
            warn!("no entry found for {}", uki);
            return ExitCode::from(0);
        };
        if let Err(e) = remove_bootentry(&bootcfg, idx, dryrun) {
            error!("remove boot entry: {}", e);
            return ExitCode::from(1);
        }
        return ExitCode::from(0);
    }

    if cfg.opt_present("boot-ok") || cfg.opt_present("boot-successful") {
        if let Err(e) = boot_successful(&bootcfg, dryrun) {
            error!("update boot order: {}", e);
            return ExitCode::from(1);
        };
        if cfg.opt_present("update-csv") {
            // reload (updated) boot config
            let bootcfg = BootConfig::new();
            if let Err(e) = update_csv(&bootcfg, efi_shim.as_ref().ok()) {
                error!("update boot.csv: {}", e);
                return ExitCode::from(1);
            }
        }
        return ExitCode::from(0);
    }

    if cfg.opt_present("update-csv") {
        if let Err(e) = update_csv(&bootcfg, efi_shim.as_ref().ok()) {
            error!("update boot.csv: {}", e);
            return ExitCode::from(1);
        }
        return ExitCode::from(0);
    }

    if let Some(uri) = cfg.opt_str("add-uri") {
        let index = bootcfg.index_unused();
        let entry_opt = create_uri_boot_entry(&uri, cfg.opt_str("title"), cfg.opt_str("cmdline"));
        let Some(entry) = entry_opt else {
            error!("failed to create boot entry for {}", uri);
            return ExitCode::from(1);
        };
        if let Err(e) = write_bootentry(&index, entry, dryrun) {
            error!("write boot entry: {}", e);
            return ExitCode::from(1);
        };
        if let Err(e) = add_bootindex(&bootcfg, &index, &cfg) {
            error!("add boot index: {}", e);
            return ExitCode::from(1);
        };
        return ExitCode::from(0);
    }

    if let Some(nr) = cfg.opt_str("remove-entry") {
        let i = u16::from_str_radix(&nr, 16);
        if let Err(e) = i {
            error!("parse \"{}\": {}", nr, e);
            return ExitCode::from(1);
        }
        let index = boot::BootIndex(i.unwrap());
        if let Err(e) = remove_bootentry(&bootcfg, &index, dryrun) {
            error!("remove boot entry: {}", e);
            return ExitCode::from(1);
        }
        return ExitCode::from(0);
    }

    if cfg.opt_present("print-loader") {
        let Some(var) = sysfs::varstore_read("LoaderInfo", &guids::LoaderInfo) else {
            println!("unknown");
            return ExitCode::from(0);
        };
        let mut r = Cursor::new(var.data);
        let Ok(name) = r.read_efi_string() else {
            println!("unknown");
            return ExitCode::from(0);
        };
        println!("{}", name);
        return ExitCode::from(0);
    }

    show_bootcfg(&bootcfg, verbose);
    ExitCode::from(0)
}
