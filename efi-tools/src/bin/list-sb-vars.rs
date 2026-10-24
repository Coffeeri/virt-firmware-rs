use clap::Parser;
use log::error;
use uguid::Guid;

use virtfw_efi_tools::cmdline::list_sb_vars::Args;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::info::CertInfo;
use virtfw_libefi::varstore::sysfs;

fn print_cert(cfg: &Args, der: &[u8]) {
    let info = CertInfo::new_from_der(der).expect("x509 cert parse error");
    if info.subject == info.issuer {
        println!("    subject: \"{}\" (self-signed)", info.desc());
    } else {
        println!("    subject: \"{}\"", info.desc());
    }
    if !cfg.verbose {
        return;
    }
    if info.subject != info.issuer {
        println!("    issuer : \"{}\"", info.desc_issuer());
    }
    println!("    valid  : {} -> {}", info.not_before, info.not_after);
    println!("    signed : {}", info.signature);
    println!("    fp     : {}", info.fingerprint());
}

fn print_sigdb(cfg: &Args, name: &str, guid: &Guid) {
    let Some(var) = sysfs::varstore_read(name, guid) else {
        println!("no {name}");
        return;
    };
    println!("{name}");

    let Some(sigdb) = EfiSigDB::new_from_bytes(var.data()) else {
        error!("parse error");
        return;
    };

    for siglist in sigdb.siglists {
        println!("  {siglist}");
        if let Some(der) = siglist.get_x509() {
            print_cert(cfg, der);
        }
    }
}

fn loginit() {
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), log::LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();
}

fn main() {
    let cfg = Args::parse();
    loginit();

    print_sigdb(&cfg, "PK", &guids::EfiGlobalVariable);
    print_sigdb(&cfg, "KEK", &guids::EfiGlobalVariable);
    print_sigdb(&cfg, "db", &guids::EfiImageSecurityDatabase);
    print_sigdb(&cfg, "dbx", &guids::EfiImageSecurityDatabase);
    print_sigdb(&cfg, "MokListRT", &guids::ShimVariable);
    print_sigdb(&cfg, "MokListXRT", &guids::ShimVariable);
}
