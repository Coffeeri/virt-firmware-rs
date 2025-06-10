use der::Decode;
use log::error;
use std::env;
use uguid::Guid;
use x509_cert::Certificate;

use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::varstore::sysfs;

fn print_sigdb(name: &str, guid: &Guid) {
    let Some(var) = sysfs::varstore_read(name, guid) else {
        println!("no {}", name);
        return;
    };
    println!("{}", name);

    let Some(sigdb) = EfiSigDB::new_from_bytes(&var.data) else {
        error!("parse error");
        return;
    };

    for siglist in sigdb.siglists {
        println!("  {}", siglist);
        if let Some(der) = siglist.get_x509() {
            if let Ok(cert) = Certificate::from_der(der) {
                println!("    subject: {}", cert.tbs_certificate.subject);
                println!("    issuer : {}", cert.tbs_certificate.issuer);
            }
        }
    }
}

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");

    let cfg_res = opts.parse(&args[1..]);
    let Ok(cfg) = cfg_res else {
        println!("{:?}", cfg_res.err());
        return None;
    };
    if cfg.opt_present("help") {
        print!("{}", opts.usage("uefi boot menu"));
        return None;
    };

    Some(cfg)
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
    let Some(_cfg) = parse_args() else {
        return;
    };
    loginit();

    print_sigdb("PK", &guids::EfiGlobalVariable);
    print_sigdb("KEK", &guids::EfiGlobalVariable);
    print_sigdb("db", &guids::EfiImageSecurityDatabase);
    print_sigdb("dbx", &guids::EfiImageSecurityDatabase);
    print_sigdb("MokListRT", &guids::ShimVariable);
    print_sigdb("MokListXRT", &guids::ShimVariable);
}
