use authenticode::{authenticode_digest, AttributeCertificateIterator};
use clap::{CommandFactory, Parser, ValueEnum};
use der::asn1::ObjectIdentifier;
use log::{error, info};
use object::pe::ImageNtHeaders64;
use object::read::pe;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::process::ExitCode;
use uguid::Guid;

use virtfw_libefi::eficms::openssl::EfiCmsOpenSSL;
use virtfw_libefi::eficms::parse::{
    get_certificate, get_issuer_cert, get_issuer_serial, get_signed_data, get_signer_info,
    preprocess_authenticode,
};
use virtfw_libefi::eficms::{EfiCms, EfiCmsCert, EfiCmsError, EfiCmsNull};
use virtfw_libefi::efivar::auth::EfiVarAuth2;
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr, EfiVarId};
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::info::CertInfo;
use virtfw_libefi::sb::certs::loadfile::cert_loadfile;
use virtfw_libefi::sb::certs::oids::{
    oid_to_name, OID_ALGO_SHA256, OID_ALGO_SHA384, OID_ALGO_SHA512,
};
use virtfw_libefi::sb::certs::profiles::{PkEnroll, SecureBootProfile};
use virtfw_libefi::sb::certs::{MICROSOFT_KEK_2011, MICROSOFT_KEK_2023};
use virtfw_libefi::varstore::sysfs;

#[derive(Debug, Clone, ValueEnum)]
pub enum EfiCmsImpl {
    #[value(name = "null", help = "dummy implementation")]
    Null,
    #[value(name = "openssl", help = "OpenSSL implementation (default)")]
    OpenSSLPkcs7,
}

#[derive(Parser, Debug)]
#[command(version, author, name = "check-sb-signature",
          about = "pkcs7 signature checker for efi binaries and efi variable updates",
          long_about = None)]
struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    loglevel: log::Level,

    /// pkcs7 verification code
    #[arg(long, hide = true)]
    code: Option<EfiCmsImpl>,

    /// select 'PK' option
    #[arg(long, value_name = "OPT", help_heading = "certificate configuration")]
    pk_enroll: Option<PkEnroll>,

    /// use default 'KEK' certificates
    #[arg(long, help_heading = "certificate configuration")]
    default_kek: bool,

    /// select 'db' profile
    #[arg(long, value_name = "NAME", help_heading = "certificate configuration")]
    profile: Option<SecureBootProfile>,

    /// load additional certificates
    #[arg(long, value_name = "FILE", help_heading = "certificate configuration")]
    cert: Vec<String>,

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
    signed_data: &[u8],
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
        println!("    subject: {}", info.desc());
        println!("    issuer : {}", info.desc_issuer());
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

fn hash_pe_binary(
    pefile: &pe::PeFile<ImageNtHeaders64>,
    algo: ObjectIdentifier,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let hash = match algo {
        OID_ALGO_SHA256 => {
            let mut hasher = Sha256::new();
            authenticode_digest(pefile, &mut hasher).expect("hash pefile");
            hasher.finalize().as_slice().into()
        }
        OID_ALGO_SHA384 => {
            let mut hasher = Sha384::new();
            authenticode_digest(pefile, &mut hasher).expect("hash pefile");
            hasher.finalize().as_slice().into()
        }
        OID_ALGO_SHA512 => {
            let mut hasher = Sha512::new();
            authenticode_digest(pefile, &mut hasher).expect("hash pefile");
            hasher.finalize().as_slice().into()
        }
        _ => return Err("unknown hash algorithm".into()),
    };

    Ok(hash)
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

    for (i, item) in iter.enumerate() {
        println!("- signature #{i}");
        let ac = item.as_ref().expect("get AttributeCertificate");
        let sig = ac
            .get_authenticode_signature()
            .expect("get AuthenticodeSignature");

        let name = oid_to_name(&sig.digest_algorithm().oid);
        println!(
            "  authenticode/algo: {} ({})",
            name,
            sig.digest_algorithm().oid
        );

        let hash = hash_pe_binary(&pefile, sig.digest_algorithm().oid)?;
        println!("  authenticode/hashed: {}", hex::encode(&hash));

        let authenticode_good = *sig.digest() == *hash;
        println!(
            "  authenticode/signed: {} {}",
            emoji(authenticode_good),
            hex::encode(sig.digest())
        );

        let (signature, content) =
            preprocess_authenticode(ac.data).expect("preprocess authenticode signature");
        match process_signature(eficms, &signature, &content) {
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
                println!("    subject: {}", info.desc());
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

    let passed = match process_signature(eficms, &signature, &signed_data) {
        Ok(r) => r,
        Err(e) => {
            error!("{e}");
            false
        }
    };

    println!("- variable update timestamp: {}", authvar.header().time);
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

fn load_certificates(cfg: &Args, eficms: &mut dyn EfiCms) {
    if let Some(ref pkenroll) = cfg.pk_enroll {
        info!("using config '{pkenroll:?}' for 'PK'");
        let pk: Vec<u8> = pkenroll.sigdb().into();
        eficms.add_certs_sigdb(EfiCmsCert::PK, &pk);
    } else if let Some(pk) = get_sigdb("PK", &guids::EfiGlobalVariable) {
        info!("using host efi variable for 'PK'");
        eficms.add_certs_sigdb(EfiCmsCert::PK, &pk);
    }

    if cfg.default_kek {
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

    if let Some(ref profile) = cfg.profile {
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

    if !cfg.cert.is_empty() {
        let certs = cfg
            .cert
            .iter()
            .map(|fname| cert_loadfile(fname).expect("load certificate"))
            .collect();
        eficms.add_certs_vec(EfiCmsCert::Other, certs);
    }
}

fn main() -> ExitCode {
    let cfg = Args::parse();
    let mut rc = 0;

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    let levelfilter = cfg.loglevel.to_level_filter();
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .filter_module("virtfw_libefi", levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let eficms: &mut dyn EfiCms = match cfg.code {
        Some(EfiCmsImpl::Null) => &mut EfiCmsNull::new(),
        Some(EfiCmsImpl::OpenSSLPkcs7) => &mut EfiCmsOpenSSL::new(),
        None => &mut EfiCmsOpenSSL::new(), // default
    };

    load_certificates(&cfg, eficms);

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
