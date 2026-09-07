use authenticode::{authenticode_digest, AttributeCertificateIterator};
use clap::{CommandFactory, Parser, ValueEnum};
use log::{error, info};
use object::read::pe;
use std::process::ExitCode;
use uguid::Guid;

use sha2::{Digest, Sha256};

use virtfw_libefi::efivar::auth::EfiVarAuth2;
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr, EfiVarId};
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::info::CertInfo;
use virtfw_libefi::sb::certs::oids::oid_to_name;
use virtfw_libefi::sb::certs::profiles::{PkEnroll, SecureBootProfile};
use virtfw_libefi::sb::certs::{MICROSOFT_KEK_2011, MICROSOFT_KEK_2023};
use virtfw_libefi::varstore::sysfs;

use virtfw_experimental::eficms::native::EfiCmsNative;
use virtfw_experimental::eficms::openssl::EfiCmsOpenSSLPkcs7;
use virtfw_experimental::eficms::parse::{
    get_certificate, get_issuer_cert, get_issuer_serial, get_signed_data, get_signer_info,
};
use virtfw_experimental::eficms::{EfiCms, EfiCmsCert, EfiCmsError, EfiCmsNull};

#[derive(Debug, Clone, ValueEnum)]
pub enum EfiCmsImpl {
    #[value(name = "null", help = "dummy implementation")]
    Null,
    #[value(
        name = "native",
        help = "native implementation (RustCrypto/x509-verify)"
    )]
    Native,
    #[value(
        name = "openssl",
        help = "OpenSSL implementation, using PKCS7_* functions (default)"
    )]
    OpenSSLPkcs7,
}

#[derive(Parser, Debug)]
#[command(version, author, name = "pkcs7",
          about = "pkcs7 playground",
          long_about = None)]
struct Args {
    /// pkcs7 verification code
    #[arg(long)]
    code: Option<EfiCmsImpl>,

    /// select 'PK' option
    #[arg(long, value_name = "OPT", help_heading = "certificate configuration")]
    pk_enroll: Option<PkEnroll>,

    /// use default 'KEK' certificates
    #[arg(long, help_heading = "certificate configuration")]
    kek_default: bool,

    /// select 'db' profile
    #[arg(long, value_name = "NAME", help_heading = "certificate configuration")]
    profile: Option<SecureBootProfile>,

    /// pe/efi binary to inspect
    #[arg(
        long,
        visible_alias = "efi",
        value_name = "FILE",
        help_heading = "verify signatures"
    )]
    pe: Vec<String>,

    /// KEK variable update to inspect
    #[arg(long, value_name = "FILE", help_heading = "verify signatures")]
    kek: Vec<String>,

    /// db variable update to inspect
    #[arg(long, value_name = "FILE", help_heading = "verify signatures")]
    db: Vec<String>,

    /// dbx variable update to inspect
    #[arg(long, value_name = "FILE", help_heading = "verify signatures")]
    dbx: Vec<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

const GOOD: &str = "✅";
const FAIL: &str = "❌";
const SKIP: &str = "⏩";

fn emoji(good: bool) -> &'static str {
    if good {
        GOOD
    } else {
        FAIL
    }
}

fn process_signature(
    eficms: &dyn EfiCms,
    signature: &[u8],
    signed_data: Option<&[u8]>,
) -> Result<bool, EfiCmsError> {
    let sd = get_signed_data(signature)?;
    let si = get_signer_info(&sd)?;
    let sid = get_issuer_serial(si)?;
    let issuer = get_issuer_cert(&sd)?;
    let mut checks_passed = false;

    println!("  signer/issuer: {}", sid.issuer);
    println!("  signer/serial: {}", sid.serial_number);
    println!(
        "  cms/digest   : {} ({})",
        oid_to_name(&si.digest_alg.oid),
        si.digest_alg.oid
    );
    println!(
        "  cms/signature: {} ({})",
        oid_to_name(&si.signature_algorithm.oid),
        si.signature_algorithm.oid
    );

    for (c, cert) in sd
        .certificates
        .as_ref()
        .unwrap()
        .0
        .iter()
        .filter_map(get_certificate)
        .enumerate()
    {
        let info = CertInfo::new(cert);
        println!("  - certificate #{c}");
        println!("    subject: {}", info.subject_cn.unwrap_or(info.subject));
        println!("    issuer : {}", info.issuer_cn.unwrap_or(info.issuer));
        println!("    signed : {}", info.signature);

        if cert != issuer {
            println!("    verify : {SKIP} skip (not the issuer certificate)");
            continue;
        }

        let vs = eficms.verify_signature(signature, signed_data);
        match &vs {
            Ok(_) => println!("    verify : {GOOD} good signature"),
            Err(e) => println!("    verify : {FAIL} signature: {e}"),
        }

        let vc = eficms.verify_certificate(signature, signed_data);
        match &vc {
            Ok(kind) => println!("    verify : {GOOD} good certificate [{kind:?}]"),
            Err(e) => println!("    verify : {FAIL} certificate: {e}"),
        }

        if vs.is_ok() && vc.is_ok() {
            checks_passed = true;
        }
    }

    Ok(checks_passed)
}

fn process_pe_binary(
    eficms: &dyn EfiCms,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("parsing efi/pe binary: {filename}");
    let binary = std::fs::read(filename)?;
    let pefile = pe::PeFile64::parse(binary.as_ref())?;
    let mut sig_pass = 0;
    let mut sig_count = 0;

    let Some(iter) = AttributeCertificateIterator::new(&pefile).expect("enumerate signatures")
    else {
        return Err("no signatures present".into());
    };

    let mut hasher = Sha256::new();
    authenticode_digest(&pefile, &mut hasher).expect("authenticode digest failed");
    let hash = hasher.finalize();
    println!("authenticode/sha256: {}", hex::encode(hash));

    for (i, item) in iter.enumerate() {
        println!("- signature #{i}");
        let ac = item.as_ref().expect("get AttributeCertificate");
        let sig = ac
            .get_authenticode_signature()
            .expect("get AuthenticodeSignature");

        let authenticode_good = *sig.digest() == *hash;
        let name = oid_to_name(&sig.digest_algorithm().oid);
        println!(
            "  authenticode/algo: {} ({})",
            name,
            sig.digest_algorithm().oid
        );
        println!(
            "  authenticode/hash: {} {}",
            emoji(authenticode_good),
            hex::encode(sig.digest())
        );

        match process_signature(eficms, ac.data, None) {
            Ok(true) => {
                if authenticode_good {
                    sig_pass += 1;
                }
            }
            Ok(false) => {}
            Err(e) => {
                error!("{e}");
            }
        }
        sig_count += 1;
    }

    if sig_pass > 0 {
        println!(
            "summary: {GOOD} {} (of {}) good signature(s)",
            sig_pass, sig_count
        );
    } else {
        println!("summary: {FAIL} only bad signatures");
    }

    if sig_pass == 0 {
        return Err("signature verification failed".into());
    }
    Ok(())
}

fn print_signature_database(data: &[u8]) -> Option<()> {
    let sigdb = EfiSigDB::new_from_bytes(data)?;

    println!("- variable update content");
    for siglist in sigdb.siglists {
        println!("  - {}", siglist);
        if let Some(cert) = siglist.get_x509() {
            if let Some(info) = CertInfo::new_from_der(cert) {
                println!("    subject: {}", info.subject_cn.unwrap_or(info.subject));
            }
        }
    }
    None
}

fn process_sb_var_update(
    eficms: &dyn EfiCms,
    variable: EfiVarId,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("parsing variable update: {filename}");
    let update = std::fs::read(filename)?;
    let authvar =
        EfiVarAuth2::new_from_bytes(&update).expect("failed to parse auth variable update");
    println!("timestamp: {}", authvar.header().time);

    let variable = EfiVar::new_with_vec(
        variable,
        EfiVarAttr::new_nv_bs_rt()
            .with_time_auth_wr_access(true)
            .with_append_write(true),
        Vec::new(),
    );

    println!("- signature");
    let signature = authvar.get_pkcs7();
    let signed_data = authvar.get_signed_data(&variable);

    let passed = match process_signature(eficms, &signature, Some(&signed_data)) {
        Ok(r) => r,
        Err(e) => {
            error!("{e}");
            false
        }
    };

    print_signature_database(authvar.data());

    if !passed {
        return Err("signature verification failed".into());
    }
    Ok(())
}

fn get_sigdb(name: &str, guid: &Guid) -> Option<Vec<u8>> {
    let var = sysfs::varstore_read(name, guid)?;
    Some(var.data().to_vec())
}

fn main() -> ExitCode {
    let cfg = Args::parse();
    let mut rc = 0;

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), log::LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let eficms: &mut dyn EfiCms = match cfg.code {
        Some(EfiCmsImpl::Null) => &mut EfiCmsNull::new(),
        Some(EfiCmsImpl::Native) => &mut EfiCmsNative::new(),
        Some(EfiCmsImpl::OpenSSLPkcs7) => &mut EfiCmsOpenSSLPkcs7::new(),
        None => &mut EfiCmsOpenSSLPkcs7::new(), // default
    };

    if let Some(pkenroll) = cfg.pk_enroll {
        info!("using config '{pkenroll:?}' for 'PK'");
        let pk: Vec<u8> = pkenroll.sigdb().into();
        eficms.add_certs_sigdb(EfiCmsCert::PK, &pk);
    } else if let Some(pk) = get_sigdb("PK", &guids::EfiGlobalVariable) {
        info!("using host efi variable for 'PK'");
        eficms.add_certs_sigdb(EfiCmsCert::PK, &pk);
    }

    if cfg.kek_default {
        info!("using microsoft certs for 'KEK'");
        let mut kek = EfiSigDB::new();
        kek.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_KEK_2011);
        kek.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_KEK_2023);
        let blob: Vec<u8> = kek.into();
        eficms.add_certs_sigdb(EfiCmsCert::KEK, &blob);
    } else if let Some(kek) = get_sigdb("KEK", &guids::EfiGlobalVariable) {
        info!("using host efi variable for 'KEK'");
        eficms.add_certs_sigdb(EfiCmsCert::KEK, &kek);
    }

    if let Some(profile) = cfg.profile {
        info!("using profile '{profile:?}' for 'db'");
        let db: Vec<u8> = profile.sigdb().into();
        eficms.add_certs_sigdb(EfiCmsCert::db, &db);
    } else if let Some(db) = get_sigdb("db", &guids::EfiImageSecurityDatabase) {
        info!("using host efi variable for 'db'");
        eficms.add_certs_sigdb(EfiCmsCert::db, &db);
    }

    if let Some(mok) = get_sigdb("MokListRT", &guids::ShimVariable) {
        info!("using host efi variable for 'MokList'");
        eficms.add_certs_sigdb(EfiCmsCert::MokList, &mok);
    }

    for pe in &cfg.pe {
        let r = process_pe_binary(eficms, pe);
        if let Err(e) = r {
            error!("{pe}: {e}");
            rc = 1;
        }
    }

    for update in &cfg.kek {
        let r = process_sb_var_update(eficms, ids::KEK.into(), update);
        if let Err(e) = r {
            error!("{update}: {e}");
            rc = 1;
        }
    }

    for update in &cfg.db {
        let r = process_sb_var_update(eficms, ids::DB.into(), update);
        if let Err(e) = r {
            error!("{update}: {e}");
            rc = 1;
        }
    }

    for update in &cfg.dbx {
        let r = process_sb_var_update(eficms, ids::DBX.into(), update);
        if let Err(e) = r {
            error!("{update}: {e}");
            rc = 1;
        }
    }

    rc.into()
}
