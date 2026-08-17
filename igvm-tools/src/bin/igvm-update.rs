//!
//! add kernel image to (ovmf) igvm
//!
use clap::{CommandFactory, Parser};
use log::{debug, error, info};
use std::process::ExitCode;

#[cfg(feature = "authenticode")]
use authenticode::authenticode_digest;
#[cfg(feature = "authenticode")]
use object::read::pe;
use sha2::{Digest, Sha256};

use virtfw_libefi::arch::EfiArch;
use virtfw_libefi::efivar::auth::esl_to_auth;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::hob::igvmdata::{EfiIgvmDataList, EfiIgvmDataType};
use virtfw_libefi::sb::certs::profiles::SecureBootProfile;
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;
use virtfw_libefi::types::EfiTime;

use igvm::IgvmFile;
use virtfw_igvm_tools::builder::Builder;
use virtfw_igvm_tools::inspect::{find_resetvector, inspect_igvm};
use virtfw_igvm_tools::ovmfmeta::{OvmfMeta, OvmfRegionType};

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-update",
          about = "add kernel to igvm file",
          long_about = None)]
struct Args {
    /// input igvm image
    #[arg(short, long, value_name = "FILE", required_unless_present = "manpage")]
    input: Option<String>,

    /// input shim image (optional)
    #[arg(short, long, value_name = "SHIM")]
    shim: Option<String>,

    /// input kernel image (can be any efi image)
    #[arg(short, long, value_name = "KERNEL")]
    kernel: Option<String>,

    /// add authenticode sha256 hashes for shim and kernel to 'db'
    #[cfg(feature = "authenticode")]
    #[arg(long, visible_alias = "add-hash")]
    add_hash_sha256: bool,

    /// add microsoft certificates to 'db'
    #[arg(long)]
    add_cert_ms: bool,

    /// select 'db' profile
    #[arg(long, value_name = "NAME")]
    profile: Option<SecureBootProfile>,

    /// select efi architecture for 'dbx'
    #[arg(long, value_name = "ARCH")]
    arch: Option<EfiArch>,

    /// inspect updated igvm
    #[arg(long)]
    inspect: bool,

    /// output igvm image
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

#[cfg(feature = "authenticode")]
fn get_sha256(binary: &[u8]) -> [u8; 32] {
    let pefile = pe::PeFile64::parse(binary).expect("efi binary parse error");
    let mut hasher = Sha256::new();
    authenticode_digest(&pefile, &mut hasher).expect("calculating authenticode hash failed");
    hasher.finalize().into()
}

fn siglist_dummy() -> Vec<[u8; 32]> {
    let hasher = Sha256::new();
    let dummy = hasher.finalize().into();
    vec![dummy]
}

fn igvm_add_kernel(
    cfg: &Args,
    igvm: &IgvmFile,
    shim_blob: Option<&[u8]>,
    kernel_blob: Option<&[u8]>,
) -> IgvmFile {
    let mut builder = Builder::from(igvm);
    let mut hoblist = EfiIgvmDataList::new(0x20000000); // start at 512 MB
    #[cfg(feature = "authenticode")]
    let mut shalist = Vec::new();

    #[cfg(feature = "authenticode")]
    let secureboot = cfg.add_hash_sha256 || cfg.add_cert_ms;

    #[cfg(not(feature = "authenticode"))]
    let secureboot = cfg.add_cert_ms;

    // shim + kernel
    #[cfg(feature = "authenticode")]
    if cfg.add_hash_sha256 {
        if let Some(b) = shim_blob {
            info!("calc shim authenticode sha256 hash");
            shalist.push(get_sha256(b));
        }
        if let Some(b) = kernel_blob {
            info!("calc kernel authenticode sha256 hash");
            shalist.push(get_sha256(b));
        }
    }

    // PK
    let mut pk = EfiSigDB::new();
    pk.add_x509_from_der(&guids::MicrosoftVendor, REDHAT_PK_KEK_2014);
    let pk_ts = pk.get_x509_mtime().unwrap();
    let pk_blob = esl_to_auth(&pk_ts, &Vec::<u8>::from(&pk));

    // KEK
    let mut kek = EfiSigDB::new();
    kek.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_KEK_2011);
    kek.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_KEK_2023);
    let kek_ts = kek.get_x509_mtime().unwrap();
    let kek_blob = esl_to_auth(&kek_ts, &Vec::<u8>::from(&kek));

    // db
    let mut db = if cfg.add_cert_ms {
        let profile = cfg.profile.as_ref().unwrap_or(&SecureBootProfile::Uefi11);
        info!("add microsoft uefi certificates to 'db' (profile: {profile:?})");
        profile.sigdb()
    } else {
        EfiSigDB::new()
    };
    #[cfg(feature = "authenticode")]
    if cfg.add_hash_sha256 && !shalist.is_empty() {
        info!("add authenticode sha256 hash(es) to 'db'");
        db.add_sha256_from_list(&guids::OvmfEnrollDefaultKeys, shalist);
    }
    if secureboot && db.siglists.is_empty() {
        info!("add dummy authenticode sha256 hash to 'db'");
        db.add_sha256_from_list(&guids::OvmfEnrollDefaultKeys, siglist_dummy());
    }
    let db_ts = if let Some(ts) = db.get_x509_mtime() {
        ts
    } else {
        // FIXME: need something better when using hashes ...
        EfiTime::initial()
    };
    let db_blob = esl_to_auth(&db_ts, &Vec::<u8>::from(&db));

    // dbx
    let dbx_blob = get_dbx_opt(cfg.arch);

    // calculate load addresses + create hobs
    if secureboot {
        info!("add secure boot signature databases");
        hoblist.add(&pk_blob, EfiIgvmDataType::Pk, true);
        hoblist.add(&kek_blob, EfiIgvmDataType::Kek, true);
        hoblist.add(&db_blob, EfiIgvmDataType::Db, true);
        hoblist.add(dbx_blob, EfiIgvmDataType::Dbx, true);
    }
    if let Some(b) = shim_blob {
        info!("add shim efi binary");
        hoblist.add(b, EfiIgvmDataType::Shim, !secureboot);
    }
    if let Some(b) = kernel_blob {
        info!("add kernel efi binary");
        hoblist.add(b, EfiIgvmDataType::Kernel, !secureboot);
    }

    let resetvector = igvm
        .directives()
        .iter()
        .filter_map(find_resetvector)
        .next()
        .expect("igvm: reset vector not found");
    let ovmfmeta = OvmfMeta::new(resetvector).expect("igvm: no ovmf metadata found");
    let hobarea = &ovmfmeta
        .regions
        .iter()
        .find(|r| r.etype == OvmfRegionType::IgvmHobArea)
        .expect("igvm: no hob area found");

    // add igvm data index hob
    if !hoblist.is_empty() {
        info!("add efidata hoblist");
        debug!("hobarea: {hobarea}");
        let hobs_blob = hoblist.hobs();
        builder.remove_page(hobarea.memory.0);
        builder.add_data_pages(hobarea.memory.0, &hobs_blob);
    }

    // add data
    for (addr, blob) in hoblist.blobs(true) {
        builder.add_data_pages(addr, blob);
    }
    for (addr, blob) in hoblist.blobs(false) {
        builder.add_data_pages_unmeasured(addr, blob);
    }

    // sanity checks
    builder.verify().expect("verify updated igvm");

    // build updated igvm
    builder.finalize().expect("build updated igvm")
}

fn process(cfg: &Args) -> Result<IgvmFile, Box<dyn std::error::Error>> {
    let input = cfg.input.as_ref().unwrap();
    info!("reading {}", input);
    let blob = std::fs::read(input)?;
    let igvm = IgvmFile::new_from_binary(&blob, None)?;

    // shim
    let shim_blob = if let Some(shim_name) = &cfg.shim {
        info!("reading {}", shim_name);
        let data = std::fs::read(shim_name)?;
        Some(data)
    } else {
        None
    };

    // kernel
    let kernel_blob = if let Some(kernel_name) = &cfg.kernel {
        info!("reading {}", kernel_name);
        let data = std::fs::read(kernel_name)?;
        Some(data)
    } else {
        None
    };

    // merge
    let new_igvm = igvm_add_kernel(cfg, &igvm, shim_blob.as_deref(), kernel_blob.as_deref());
    if cfg.inspect {
        println!("updated IGVM image");
        inspect_igvm(&new_igvm);
    }

    Ok(new_igvm)
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    let levelfilter = log::LevelFilter::Debug;
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .filter_module("virtfw_igvm_tools::efidata", levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let igvm = match process(&cfg) {
        Err(e) => {
            error!("{e}");
            return 1.into();
        }
        Ok(i) => i,
    };

    if let Some(outfile) = cfg.output {
        let mut blob = Vec::new();
        let res = igvm.serialize(&mut blob);
        if let Err(e) = res {
            error!("serialize igvm: {e}");
            return ExitCode::from(1);
        }

        info!("writing {}", outfile);
        let res = std::fs::write(&outfile, blob);
        if let Err(e) = res {
            error!("write {}: {}", outfile, e);
            return ExitCode::from(1);
        }
    } else {
        info!("igvm ok");
    }

    ExitCode::from(0)
}
