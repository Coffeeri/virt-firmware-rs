use clap::{Parser, ValueEnum};
use log::error;
use std::process::ExitCode;

use der::{DecodePem, Encode};
use x509_cert::Certificate;

use virtfw_libefi::arch::EfiArch;
use virtfw_libefi::efivar::auth::auth_to_esl;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::profiles::SecureBootProfile;
use virtfw_libefi::sb::dbx::*;
use virtfw_varstore::store::EfiVarStore;

#[derive(Debug, Clone, ValueEnum)]
pub enum PkEnroll {
    #[value(name = "mgmt", help = "EFI_CERT_EXTERNAL_MANAGEMENT_GUID")]
    ExtMgmt,
    #[value(name = "redhat", help = "Red Hat Secure Boot (PK/KEK key 1)")]
    RedHat,
}

#[derive(Parser, Debug)]
#[command(version, name = "virt-fw-vars-setup",
          about = "generate uefi variable store in json format",
          long_about = None)]
struct Args {
    /// json variable store output file
    ///
    /// If not specified output goes to stdout.
    #[arg(long, value_name = "FILE")]
    output: Option<String>,

    /// select 'db' profile
    ///
    /// Use 'uefi*' profiles for linux guests, use 'win*' profiles for
    /// windows guests.  If in doubt you can also use the 'all'
    /// profile which includes both and matches the typical
    /// configuration of physical hardware.
    #[arg(long, value_name = "NAME", help_heading = "Quick setup")]
    profile: SecureBootProfile,

    /// select 'PK' mode
    #[arg(long, value_name = "MODE", help_heading = "Quick setup",
          value_enum, default_value_t = PkEnroll::ExtMgmt)]
    enroll: PkEnroll,

    /// select efi architecture for 'dbx' update
    ///
    /// By default native architecture will be used.
    #[arg(long, value_name = "ARCH", help_heading = "Quick setup")]
    arch: Option<EfiArch>,

    /// set certificate for 'PK'
    ///
    /// This has higher priority than PK mode.
    #[arg(long, value_name = "FILE", help_heading = "Specify details")]
    set_pk_cert: Option<String>,

    /// add certificate to 'KEK'
    ///
    /// Can be specified multiple times.  Default is to enroll the
    /// standard microsoft KEK certificates.
    #[arg(long, value_name = "FILE", help_heading = "Specify details")]
    add_kek_cert: Vec<String>,

    /// add certificate to 'db'
    ///
    /// Can be specified multiple times.  Certificates will be
    /// enrolled in addition to the profile certificates.  Use
    /// '--profile none' if you want specify all certificates
    /// explicitly.
    #[arg(long, value_name = "FILE", help_heading = "Specify details")]
    add_db_cert: Vec<String>,

    /// load 'dbx' update from file
    ///
    /// If specified replaces the compiled-in default update.
    #[arg(long, value_name = "FILE", help_heading = "Specify details")]
    set_dbx_update: Option<String>,

    /// disable secure boot
    ///
    /// Enroll all secure boot variables but disable secure boot.
    #[arg(long, help_heading = "Specify details")]
    disable_secure_boot: bool,
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

fn cert_load(filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let data = std::fs::read(filename)?;

    if cert_is_pem(&data) {
        let cert = Certificate::from_pem(&data)?;
        Ok(cert.to_der().expect("serialize cert"))
    } else {
        Ok(data)
    }
}

fn main() -> ExitCode {
    let cfg = Args::parse();
    loginit();

    let mut store = EfiVarStore::new();

    // db
    let mut db = cfg.profile.sigdb();
    for filename in cfg.add_db_cert {
        let der = cert_load(&filename);
        if let Err(e) = der {
            error!("{filename}: {e}");
            return 1.into();
        }
        db.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der.unwrap());
    }
    store.enroll_db(db.get_x509_mtime(), &db);

    // dbx
    let auth = if let Some(filename) = cfg.set_dbx_update {
        match std::fs::read(&filename) {
            Err(e) => {
                error!("{filename}: {e}");
                return 1.into();
            }
            Ok(a) => a,
        }
    } else {
        get_dbx_opt(cfg.arch).into()
    };
    let (dbx_ts, esl) = auth_to_esl(&auth).unwrap();
    let dbx = EfiSigDB::new_from_bytes(esl).unwrap();
    store.enroll_dbx(Some(dbx_ts), &dbx);

    // KEK
    if cfg.add_kek_cert.is_empty() {
        store.enroll_kek_microsoft();
    } else {
        let mut kek = EfiSigDB::new();
        for filename in cfg.add_kek_cert {
            let der = cert_load(&filename);
            if let Err(e) = der {
                error!("{filename}: {e}");
                return 1.into();
            }
            kek.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der.unwrap());
        }
        store.enroll_kek(kek.get_x509_mtime(), &kek);
    }

    // PK
    if let Some(filename) = cfg.set_pk_cert {
        let mut pk = EfiSigDB::new();
        let der = cert_load(&filename);
        if let Err(e) = der {
            error!("{filename}: {e}");
            return 1.into();
        }
        pk.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der.unwrap());
        store.enroll_pk(pk.get_x509_mtime(), &pk);
    } else {
        match cfg.enroll {
            PkEnroll::RedHat => store.enroll_pk_redhat(),
            PkEnroll::ExtMgmt => store.enroll_pk_mgmt(),
        }
    }

    // SecureBootEnable
    store.set_secure_boot_enable(!cfg.disable_secure_boot);

    let jstore = store.json_export();
    let json = serde_json::to_string(&jstore).expect("serialize to json failed");

    if let Some(filename) = cfg.output {
        if let Err(e) = std::fs::write(&filename, json) {
            error!("{filename}: {e}");
            return 1.into();
        }
    } else {
        println!("{json}");
    };

    0.into()
}
