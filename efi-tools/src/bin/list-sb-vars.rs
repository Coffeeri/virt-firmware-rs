use clap::Parser;
use log::error;
use uguid::Guid;

use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::info::CertInfo;
use virtfw_libefi::varstore::sysfs;

#[derive(Parser, Debug)]
#[command(version, name = "list-sb-vars", about = "list secure boot databases", long_about = None)]
struct Args {}

fn print_cert(der: &[u8]) {
    let info = CertInfo::new_from_der(der).expect("x509 cert parse error");
    if info.subject == info.issuer {
        println!(
            "    subject: {} (self-signed)",
            info.subject_cn.unwrap_or("no CN".to_string())
        );
    } else {
        println!(
            "    subject: {}",
            info.subject_cn.unwrap_or("no CN".to_string())
        );
        println!(
            "    issuer : {}",
            info.issuer_cn.unwrap_or("no CN".to_string())
        );
    }
    println!("    valid  : {} -> {}", info.not_before, info.not_after);
    println!("    signed : {}", info.signature);
}

fn print_sigdb(name: &str, guid: &Guid) {
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
            print_cert(der);
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
    Args::parse();
    loginit();

    print_sigdb("PK", &guids::EfiGlobalVariable);
    print_sigdb("KEK", &guids::EfiGlobalVariable);
    print_sigdb("db", &guids::EfiImageSecurityDatabase);
    print_sigdb("dbx", &guids::EfiImageSecurityDatabase);
    print_sigdb("MokListRT", &guids::ShimVariable);
    print_sigdb("MokListXRT", &guids::ShimVariable);
}
