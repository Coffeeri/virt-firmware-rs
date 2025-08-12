use clap::Parser;
use log::{debug, error, info, warn};
use std::io;
use std::io::Cursor;
use std::process::ExitCode;
use std::str::FromStr;

use virtfw_libefi::bootcfg::BootConfig;
use virtfw_libefi::distro::LinuxOsInfo;
use virtfw_libefi::efifile::EfiFile;
use virtfw_libefi::efivar::boot;
use virtfw_libefi::efivar::devpath as dp;
use virtfw_libefi::efivar::io::ReadEfiExt;
use virtfw_libefi::efivar::io::WriteEfiExt;
use virtfw_libefi::efivar::types::EfiVar;
use virtfw_libefi::efivar::types::EfiVarAttr;
use virtfw_libefi::guids;
use virtfw_libefi::nocasestr::NoCaseString;
use virtfw_libefi::varstore::sysfs;

use virtfw_efi_tools::shimcsv::write_boot_csv;

#[derive(Parser, Debug)]
#[command(version, about = "bootcfg - show and manage uefi boot entries", long_about = None)]
struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL")]
    loglevel: Option<String>,

    /// print boot configuration
    #[arg(short, long)]
    show: bool,

    /// more verbose output
    #[arg(short, long)]
    verbose: bool,

    /// add boot entry for UKI image FILE
    #[arg(long, value_name = "FILE")]
    add_uki: Option<String>,

    /// remove boot entry for UKI image FILE
    #[arg(long, value_name = "FILE")]
    remove_uki: Option<String>,

    /// override cmd line for boot entry
    #[arg(long, value_name = "CMDLINE")]
    cmdline: Option<String>,

    /// boot is successful.
    #[arg(long = "boot-ok", visible_alias = "boot-successful")]
    boot_successful: bool,

    /// update BOOT.CSV
    #[arg(long)]
    update_csv: bool,

    /// add boot entry to netboot URI
    #[arg(long, value_name = "URI")]
    add_uri: Option<String>,

    /// remove boot entry NNNN
    #[arg(long, value_name = "NNNN")]
    remove_entry: Option<String>,

    /// boot added entry once (via BootNext)
    #[arg(long = "boot-next", visible_alias = "once")]
    boot_next: bool,

    /// place added entry at POS to BootOrder
    #[arg(long, value_name = "POS")]
    boot_order: Option<String>,

    /// do not actually update the configuration
    #[arg(long)]
    dry_run: bool,

    /// set title for boot entry
    #[arg(long, value_name = "TITLE")]
    title: Option<String>,

    /// print boot loader name
    #[arg(long)]
    print_loader: bool,
}

fn create_file_boot_entry(
    bootcfg: &BootConfig,
    shim_opt: Option<&EfiFile>,
    uki: &EfiFile,
    title: Option<&String>,
    cmdline: Option<&String>,
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
            Some(c) => Some(format!("{efipath} {c}")),
            None => Some(efipath.0),
        }
    } else {
        if bootcfg.sb {
            error!("shim not found and secure boot enabled");
            return None;
        }
        devpath = uki.devpath();
        optdata = cmdline.map(|c| c.to_string());
    };

    let t = match title {
        Some(t) => t.clone(),
        None => uki.basename().unwrap(),
    };

    Some(boot::BootEntry::new_boot(&t, devpath.unwrap(), optdata))
}

fn create_uri_boot_entry(
    uri: &str,
    title: Option<&String>,
    cmdline: Option<&String>,
) -> Option<boot::BootEntry> {
    let mut devpath = dp::DevPath::new();
    devpath.push(dp::DevPathNode::MsgUri(Some(uri.to_string())));
    let optdata = cmdline.map(|c| c.to_string());
    let t = match title {
        Some(t) => t.clone(),
        None => uri.split('/').next_back().unwrap().to_string(),
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
        info!("skip delete efivar: {name} (dry-run)");
    } else {
        info!("delete efivar: {name}");
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

    let name = format!("Boot{index}");
    let be = EfiVar::new_with_vec(
        &guids::EfiGlobalVariable,
        &name,
        EfiVarAttr::new_nv_bs_rt(),
        wr.into_inner(),
    );

    info!(
        "set efivar: {} = \"{}\" {} {}",
        be.name, entry.title, entry.devpath, entry.optdata
    );
    write_variable(&be, dryrun)?;
    Ok(())
}

fn write_bootnext(index: &boot::BootIndex, dryrun: bool) -> io::Result<()> {
    let bn = EfiVar::new_with_vec(
        &guids::EfiGlobalVariable,
        "BootNext",
        EfiVarAttr::new_nv_bs_rt(),
        Vec::from(index),
    );

    debug!("set efivar: {} = {}", bn.name, index);
    write_variable(&bn, dryrun)?;
    Ok(())
}

fn update_bootorder(
    new_order: &boot::BootOrder,
    old_order: &boot::BootOrder,
    dryrun: bool,
) -> io::Result<()> {
    let bo = EfiVar::new_with_vec(
        &guids::EfiGlobalVariable,
        "BootOrder",
        EfiVarAttr::new_nv_bs_rt(),
        Vec::from(new_order),
    );

    debug!("old efivar: {} = {}", bo.name, old_order);
    debug!("set efivar: {} = {}", bo.name, new_order);
    if new_order.is_empty() {
        delete_variable("BootOrder", dryrun)?;
    } else {
        write_variable(&bo, dryrun)?;
    }
    Ok(())
}

fn add_bootindex(bootcfg: &BootConfig, index: &boot::BootIndex, cfg: &Args) -> io::Result<()> {
    let dryrun = cfg.dry_run;
    if cfg.boot_next {
        write_bootnext(index, dryrun)?;
    }
    if let Some(posstr) = &cfg.boot_order {
        let old_order = if let Some(o) = &bootcfg.order {
            o
        } else {
            &boot::BootOrder::empty()
        };

        let pos = if posstr == "last" {
            old_order.len()
        } else {
            let Ok(p) = posstr.parse::<usize>() else {
                return Err(io::Error::other(format!("failed to parse pos: {posstr}")));
            };
            if p > old_order.len() {
                return Err(io::Error::other(format!(
                    "pos is too big: {} > {}",
                    p,
                    old_order.len()
                )));
            }
            p
        };

        let new_order = old_order.insert(pos, index);
        update_bootorder(&new_order, old_order, dryrun)?;
    }
    Ok(())
}

fn remove_bootentry(bootcfg: &BootConfig, index: &boot::BootIndex, dryrun: bool) -> io::Result<()> {
    let name = format!("Boot{index}");
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
        info!("BootCurrent ({current}) already in BootOrder (position {pos})");
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
        debug!("{line}");
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
                        println!("      optdata/string: \"{string}\"")
                    }
                    boot::BootEntryOptData::Guid { guid } => {
                        println!("      optdata/guid: {guid}")
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

fn main() -> ExitCode {
    let cfg = Args::parse();

    let deflevel = stderrlog::LogLevelNum::Info;
    let loglevel = if let Some(s) = cfg.loglevel.as_ref() {
        if let Ok(l) = log::Level::from_str(s) {
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

    let dryrun = cfg.dry_run;
    let verbose = cfg.verbose;

    let osinfo = LinuxOsInfo::new();
    let bootcfg = BootConfig::new();

    let shim = osinfo.shim_path().unwrap();
    let efi_shim = EfiFile::new(&shim);

    if cfg.show {
        show_bootcfg(&bootcfg, verbose);
        return ExitCode::from(0);
    }

    if let Some(ref uki) = cfg.add_uki {
        let efi_uki = match EfiFile::new(uki) {
            Err(e) => {
                error!("{uki}: {e}");
                return ExitCode::from(1);
            }
            Ok(u) => u,
        };

        let idx_opt = find_boot_entry_for_file(&bootcfg, efi_shim.as_ref().ok(), &efi_uki);
        let index = match idx_opt {
            Some(idx) => {
                info!("entry exists for {uki} (Boot{idx})");
                idx
            }
            None => &bootcfg.index_unused(),
        };
        let entry_opt = create_file_boot_entry(
            &bootcfg,
            efi_shim.as_ref().ok(),
            &efi_uki,
            cfg.title.as_ref(),
            cfg.cmdline.as_ref(),
        );
        let Some(entry) = entry_opt else {
            error!("failed to create boot entry for {uki}");
            return ExitCode::from(1);
        };
        if let Err(e) = write_bootentry(index, entry, dryrun) {
            error!("write boot entry: {e}");
            return ExitCode::from(1);
        };
        if let Err(e) = add_bootindex(&bootcfg, index, &cfg) {
            error!("add boot index: {e}");
            return ExitCode::from(1);
        };
        return ExitCode::from(0);
    }

    if let Some(ref uki) = cfg.remove_uki {
        let efi_uki = match EfiFile::new(uki) {
            Err(e) => {
                error!("{uki}: {e}");
                return ExitCode::from(1);
            }
            Ok(u) => u,
        };

        let idx_opt = find_boot_entry_for_file(&bootcfg, efi_shim.as_ref().ok(), &efi_uki);
        let Some(idx) = idx_opt else {
            warn!("no entry found for {uki}");
            return ExitCode::from(0);
        };
        if let Err(e) = remove_bootentry(&bootcfg, idx, dryrun) {
            error!("remove boot entry: {e}");
            return ExitCode::from(1);
        }
        return ExitCode::from(0);
    }

    if cfg.boot_successful {
        if let Err(e) = boot_successful(&bootcfg, dryrun) {
            error!("update boot order: {e}");
            return ExitCode::from(1);
        };
        if cfg.update_csv {
            // reload (updated) boot config
            let bootcfg = BootConfig::new();
            if let Err(e) = update_csv(&bootcfg, efi_shim.as_ref().ok()) {
                error!("update boot.csv: {e}");
                return ExitCode::from(1);
            }
        }
        return ExitCode::from(0);
    }

    if cfg.update_csv {
        if let Err(e) = update_csv(&bootcfg, efi_shim.as_ref().ok()) {
            error!("update boot.csv: {e}");
            return ExitCode::from(1);
        }
        return ExitCode::from(0);
    }

    if let Some(ref uri) = cfg.add_uri {
        let index = bootcfg.index_unused();
        let entry_opt = create_uri_boot_entry(uri, cfg.title.as_ref(), cfg.cmdline.as_ref());
        let Some(entry) = entry_opt else {
            error!("failed to create boot entry for {uri}");
            return ExitCode::from(1);
        };
        if let Err(e) = write_bootentry(&index, entry, dryrun) {
            error!("write boot entry: {e}");
            return ExitCode::from(1);
        };
        if let Err(e) = add_bootindex(&bootcfg, &index, &cfg) {
            error!("add boot index: {e}");
            return ExitCode::from(1);
        };
        return ExitCode::from(0);
    }

    if let Some(ref nr) = cfg.remove_entry {
        let i = u16::from_str_radix(nr, 16);
        if let Err(e) = i {
            error!("parse \"{nr}\": {e}");
            return ExitCode::from(1);
        }
        let index = boot::BootIndex(i.unwrap());
        if let Err(e) = remove_bootentry(&bootcfg, &index, dryrun) {
            error!("remove boot entry: {e}");
            return ExitCode::from(1);
        }
        return ExitCode::from(0);
    }

    if cfg.print_loader {
        let Some(var) = sysfs::varstore_read("LoaderInfo", &guids::LoaderInfo) else {
            println!("unknown");
            return ExitCode::from(0);
        };
        let mut r = Cursor::new(var.data);
        let Ok(name) = r.read_efi_string() else {
            println!("unknown");
            return ExitCode::from(0);
        };
        println!("{name}");
        return ExitCode::from(0);
    }

    show_bootcfg(&bootcfg, verbose);
    ExitCode::from(0)
}
