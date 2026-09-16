use clap::{CommandFactory, Parser};
use log::error;
use std::process::ExitCode;

use virtfw_libefi::arch::EfiArch;
use virtfw_libefi::efivar::auth::auth_to_esl;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::loadfile::cert_loadfile;
use virtfw_libefi::sb::certs::profiles::{PkEnroll, SecureBootProfile};
use virtfw_libefi::sb::dbx::*;
use virtfw_libefi::types::EfiTime;
use virtfw_varstore::store::EfiVarStore;

#[derive(Parser, Debug)]
#[command(version, author, name = "virt-fw-vars-setup",
          about = "generate uefi variable store in json format",
          long_about = None)]
struct Args {
    /// json variable store output file
    ///
    /// If not specified output goes to stdout.
    #[arg(long, value_name = "FILE")]
    output: Option<String>,

    #[arg(long, hide = true)]
    manpage: bool,

    /// select 'db' certificate profile
    ///
    /// Use 'uefi*' profiles for linux guests, use 'win*' profiles for
    /// windows guests.  If in doubt you can also use the 'all'
    /// profile which includes both and matches the typical
    /// configuration of physical hardware.
    #[arg(
        long,
        value_name = "NAME",
        help_heading = "Quick setup",
        required_unless_present = "manpage"
    )]
    profile: Option<SecureBootProfile>,

    /// select 'PK' enroll mode
    #[arg(long, value_name = "MODE", help_heading = "Quick setup",
          value_enum, default_value_t = PkEnroll::RedHat)]
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
    let levelfilter = log::LevelFilter::Info;
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();
}

fn main() -> ExitCode {
    let cfg = Args::parse();
    loginit();

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    let mut store = EfiVarStore::new();

    // db
    let mut db = cfg.profile.unwrap().sigdb();
    for filename in cfg.add_db_cert {
        let der = cert_loadfile(&filename);
        if let Err(e) = der {
            error!("{filename}: {e}");
            return 1.into();
        }
        db.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der.unwrap());
    }
    store.enroll_db(db.get_x509_mtime(), &db);

    // dbx
    let (ts, dbx) = if let Some(filename) = cfg.set_dbx_update {
        let auth = match std::fs::read(&filename) {
            Err(e) => {
                error!("{filename}: {e}");
                return 1.into();
            }
            Ok(a) => a,
        };
        let (ts, esl) = auth_to_esl(&auth).unwrap();
        let sigdb = EfiSigDB::new_from_bytes(esl).unwrap();
        (ts, sigdb)
    } else {
        let sigdb = get_dbx_opt(cfg.arch);
        (EfiTime::dbxupdate(), sigdb)
    };
    store.enroll_dbx(Some(ts), &dbx);

    // KEK
    if cfg.add_kek_cert.is_empty() {
        let kek = cfg.enroll.kek_sigdb();
        let ts = kek.get_x509_mtime();
        store.enroll_kek(ts, &kek);
    } else {
        let mut kek = EfiSigDB::new();
        for filename in cfg.add_kek_cert {
            let der = cert_loadfile(&filename);
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
        let der = cert_loadfile(&filename);
        if let Err(e) = der {
            error!("{filename}: {e}");
            return 1.into();
        }
        pk.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der.unwrap());
        store.enroll_pk(pk.get_x509_mtime(), &pk);
    } else {
        let pk = cfg.enroll.sigdb();
        let ts = pk.get_x509_mtime();
        store.enroll_pk(ts, &pk);
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
