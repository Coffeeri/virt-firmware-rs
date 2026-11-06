//!
//! add kernel image to (ovmf) igvm
//!
use clap::Parser;
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
use virtfw_libefi::sb::certs::profiles::{PkEnroll, SecureBootProfile};
use virtfw_libefi::sb::dbx::*;
use virtfw_libefi::types::EfiTime;

use igvm::IgvmFile;
use virtfw_igvm_tools::builder::Builder;
use virtfw_igvm_tools::cmdline::igvm_update::Args;
use virtfw_igvm_tools::inspect::{find_resetvector, inspect_igvm};
use virtfw_igvm_tools::ovmfmeta::{OvmfMeta, OvmfRegionType};

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
    igvm: &IgvmFile,
    shim_blob: Option<&[u8]>,
    kernel_blob: Option<&[u8]>,
    efi_arch: Option<EfiArch>,
    sb_profile: Option<&SecureBootProfile>,
    pk_enroll: &PkEnroll,
    add_hash_sha256: bool,
) -> Result<IgvmFile, Box<dyn core::error::Error>> {
    let mut builder = Builder::from(igvm);
    let mut hoblist = EfiIgvmDataList::new(0x20000000); // start at 512 MB
    let secureboot = add_hash_sha256 || sb_profile.is_some();

    #[cfg(feature = "authenticode")]
    let mut shalist = Vec::new();

    // shim + kernel
    #[cfg(feature = "authenticode")]
    if add_hash_sha256 {
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
    let pk = pk_enroll.sigdb();
    let pk_ts = pk.get_x509_mtime().unwrap();
    let pk_blob = esl_to_auth(&pk_ts, &Vec::<u8>::from(&pk));

    // KEK
    let kek = pk_enroll.kek_sigdb();
    let kek_ts = kek.get_x509_mtime().unwrap();
    let kek_blob = esl_to_auth(&kek_ts, &Vec::<u8>::from(&kek));

    // db
    let mut db = if let Some(profile) = sb_profile {
        info!("add microsoft uefi certificates to 'db' (profile: {profile:?})");
        profile.sigdb()
    } else {
        EfiSigDB::new()
    };
    #[cfg(feature = "authenticode")]
    if add_hash_sha256 && !shalist.is_empty() {
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
    let dbx_esl: Vec<u8> = get_dbx_opt(efi_arch).into();
    let dbx_blob = esl_to_auth(&EfiTime::dbxupdate(), &dbx_esl);

    // calculate load addresses + create hobs
    if secureboot {
        info!("add secure boot signature databases");
        hoblist.add(&pk_blob, EfiIgvmDataType::Pk, true);
        hoblist.add(&kek_blob, EfiIgvmDataType::Kek, true);
        hoblist.add(&db_blob, EfiIgvmDataType::Db, true);
        hoblist.add(&dbx_blob, EfiIgvmDataType::Dbx, true);
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
    builder.verify()?;

    // build updated igvm
    let new_igvm = builder.finalize()?;

    // all done
    Ok(new_igvm)
}

fn process(cfg: &Args) -> Result<IgvmFile, Box<dyn std::error::Error>> {
    let input = &cfg.input;
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
    let sb_profile = if cfg.add_cert_ms {
        Some(cfg.profile.as_ref().unwrap_or(&SecureBootProfile::Uefi11))
    } else {
        None
    };

    #[cfg(feature = "authenticode")]
    let add_hash = cfg.add_hash_sha256;

    #[cfg(not(feature = "authenticode"))]
    let add_hash = false;

    let new_igvm = igvm_add_kernel(
        &igvm,
        shim_blob.as_deref(),
        kernel_blob.as_deref(),
        cfg.arch,
        sb_profile,
        &cfg.enroll,
        add_hash,
    )?;
    if cfg.inspect {
        println!("updated IGVM image");
        inspect_igvm(&new_igvm);
    }

    Ok(new_igvm)
}

fn main() -> ExitCode {
    let cfg = Args::parse();

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
