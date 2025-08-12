use clap::Parser;
use der::Decode;
use log::error;
use uguid::Guid;
use x509_cert::Certificate;

use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::varstore::sysfs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

fn print_cert(cert: &Certificate) {
    let tbs = &cert.tbs_certificate;
    if tbs.subject == tbs.issuer {
        println!("    subject: {} (self-signed)", tbs.subject);
    } else {
        println!("    subject: {}", tbs.subject);
        println!("    issuer : {}", tbs.issuer);
    }
    println!(
        "    valid  : {} -> {}",
        tbs.validity.not_before, tbs.validity.not_after
    );
}

fn print_sigdb(name: &str, guid: &Guid) {
    let Some(var) = sysfs::varstore_read(name, guid) else {
        println!("no {name}");
        return;
    };
    println!("{name}");

    let Some(sigdb) = EfiSigDB::new_from_bytes(&var.data) else {
        error!("parse error");
        return;
    };

    for siglist in sigdb.siglists {
        println!("  {siglist}");
        if let Some(der) = siglist.get_x509() {
            if let Ok(cert) = Certificate::from_der(der) {
                print_cert(&cert);
            }
        }
    }
}

fn loginit() {
    stderrlog::new()
        .module(module_path!())
        .module("libefi")
        .verbosity(stderrlog::LogLevelNum::Debug)
        .init()
        .unwrap();
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
