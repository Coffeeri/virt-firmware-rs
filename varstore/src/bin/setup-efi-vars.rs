use clap::Parser;

use der::{DecodePem, Encode};
use x509_cert::Certificate;

use virtfw_libefi::arch::EfiArch;
use virtfw_libefi::efivar::auth::auth_to_esl;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::profiles::SecureBootProfile;
use virtfw_libefi::sb::dbx::*;
use virtfw_varstore::store::EfiVarStore;

#[derive(Parser, Debug)]
#[command(version, about = "generate uefi variable store in json format", long_about = None)]
struct Args {
    /// select 'db' profile (none, win11, win23, uefi11, uefi23, all)
    #[arg(long, value_name = "NAME")]
    profile: SecureBootProfile,

    /// select efi architecture (x64, aa64)
    #[arg(long, value_name = "ARCH")]
    arch: Option<EfiArch>,

    /// add certificate to 'db'
    #[arg(long, value_name = "CERT")]
    db_cert: Vec<String>,
}

fn loginit() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();
}

fn cert_is_pem(data: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(data) else {
        return false;
    };
    s.contains("-----BEGIN")
}

fn cert_load(filename: &str) -> Vec<u8> {
    let data = std::fs::read(filename).expect("read cert file");

    if cert_is_pem(&data) {
        let cert = Certificate::from_pem(&data).expect("parse pem cert");
        cert.to_der().expect("serialize cert")
    } else {
        data
    }
}

fn main() {
    let cfg = Args::parse();
    loginit();

    let mut store = EfiVarStore::new();

    let mut db = cfg.profile.sigdb();

    for filename in cfg.db_cert {
        let der = cert_load(&filename);
        db.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der);
    }

    let auth = get_dbx_opt(cfg.arch);
    let (dbx_ts, esl) = auth_to_esl(auth).unwrap();
    let dbx = EfiSigDB::new_from_bytes(esl).unwrap();

    store.enroll_db(db.get_x509_mtime(), &db);
    store.enroll_dbx(Some(dbx_ts), &dbx);
    store.enroll_kek_microsoft();
    store.enroll_pk_mgmt();

    let jstore = store.json_export();
    let json = serde_json::to_string(&jstore).expect("serialize to json failed");
    println!("{json}");
}
