use clap::Parser;
use std::str::FromStr;

use der::{DecodePem, Encode};
use x509_cert::Certificate;

use virtfw_libefi::efivar::auth::auth_to_esl;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;
use virtfw_varstore::store::EfiVarStore;

#[derive(Debug, Clone)]
enum Profile {
    None,
    Win11,
    Win23,
    Uefi11,
    Uefi23,
    All,
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "win11" => Ok(Self::Win11),
            "win23" => Ok(Self::Win23),
            "uefi11" => Ok(Self::Uefi11),
            "uefi23" => Ok(Self::Uefi23),
            "all" => Ok(Self::All),
            _ => Err(format!("unknown profile: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
enum EfiArch {
    X64,
    Aa64,
}

impl FromStr for EfiArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x64" | "x86_64" => Ok(Self::X64),
            "aa64" | "aarch64" => Ok(Self::Aa64),
            _ => Err(format!("unknown arch: {s}")),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "generate uefi variable store in json format", long_about = None)]
struct Args {
    /// select 'db' profile (none, win11, win23, uefi11, uefi23, all)
    #[arg(long, value_name = "NAME")]
    profile: Profile,

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
    let Ok(s) = std::str::from_utf8(&data) else {
        return false;
    };
    s.find("-----BEGIN").is_some()
}

fn cert_load(filename: &str) -> Vec<u8> {
    let data = std::fs::read(filename).expect("read cert file");

    if cert_is_pem(&data) {
        let cert = Certificate::from_pem(&data).expect("parse pem cert");
        let der = cert.to_der().expect("serialize cert");
        der
    } else {
        data
    }
}

fn main() {
    let cfg = Args::parse();
    loginit();

    let mut store = EfiVarStore::new();

    let mut db = EfiSigDB::new();
    match cfg.profile {
        Profile::None => {}
        Profile::Win11 => {
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_WINDOWS_2011);
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
        }
        Profile::Win23 => {
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
        }
        Profile::Uefi11 => {
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
        }
        Profile::Uefi23 => {
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
        }
        Profile::All => {
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_WINDOWS_2011);
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
            db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
        }
    }

    for filename in cfg.db_cert {
        let der = cert_load(&filename);
        db.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der);
    }

    let auth = match cfg.arch {
        Some(EfiArch::X64) => Some(DBX_X86_64),
        Some(EfiArch::Aa64) => Some(DBX_AARCH64),
        None => DBX_NATIVE,
    };
    let (dbx_ts, esl) = auth_to_esl(auth.expect("dbx missing")).unwrap();
    let dbx = EfiSigDB::new_from_bytes(esl).unwrap();

    store.enroll_db(db.get_x509_mtime(), &db);
    store.enroll_dbx(Some(dbx_ts), &dbx);
    store.enroll_kek_microsoft();
    store.enroll_pk_mgmt();

    let jstore = store.json_export();
    let json = serde_json::to_string(&jstore).expect("serialize to json failed");
    println!("{json}");
}
