use authenticode::AttributeCertificateIterator;
use clap::Parser;
use der::Decode;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use log::{debug, error, info};
use object::read::pe;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use uguid::Guid;
use x509_cert::name::RdnSequence;
use x509_cert::Certificate;

use virtfw_efi_tools::cmdline::enroll_sb_vars::Args;
use virtfw_libefi::bootcfg::BootConfig;
use virtfw_libefi::eficms::parse::get_issuer_serial;
use virtfw_libefi::efifile::EfiFile;
use virtfw_libefi::efivar::auth::esl_to_auth;
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::info::CertInfo;
use virtfw_libefi::sb::certs::loadfile::cert_loadfile;
use virtfw_libefi::sb::certs::{
    MICROSOFT_DB_UEFI_2011, MICROSOFT_DB_UEFI_2023, MICROSOFT_DB_UEFI_ROM_2023,
    MICROSOFT_DB_WINDOWS_2011, MICROSOFT_DB_WINDOWS_2023, REDHAT_DB_UEFI_2024,
};
use virtfw_libefi::sb::dbx::get_dbx_native;
use virtfw_libefi::types::EfiTime;
use virtfw_libefi::varstore::sysfs;

fn loginit(cfg: &Args) {
    let levelfilter = cfg.loglevel.to_level_filter();
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .filter_module("virtfw_libefi", levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();
}

fn check_virtualization() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("systemd-detect-virt").arg("--vm").output()?;
    let virt = std::str::from_utf8(&output.stdout)?.trim();
    info!("virtualization is {virt}");
    if virt != "kvm" {
        return Err("supporting only virtual machines".into());
    }
    Ok(())
}

fn check_setup_mode() -> Result<(), Box<dyn std::error::Error>> {
    let None = sysfs::varstore_read_id(&ids::PK.into()) else {
        return Err("PK is already enrolled".into());
    };
    let None = sysfs::varstore_read_id(&ids::KEK.into()) else {
        return Err("KEK is already enrolled".into());
    };
    let None = sysfs::varstore_read_id(&ids::DB.into()) else {
        return Err("db is already enrolled".into());
    };
    let None = sysfs::varstore_read_id(&ids::DBX.into()) else {
        return Err("dbx is already enrolled".into());
    };

    let Some(sb) = sysfs::varstore_read_id(&ids::SECURE_BOOT.into()) else {
        return Err("SecureBoot variable not present".into());
    };
    let Some(sm) = sysfs::varstore_read_id(&ids::SETUP_MODE.into()) else {
        return Err("SetupMode variable not present".into());
    };
    if sb.data()[0] != 0 {
        return Err("secure boot is already enabled".into());
    }
    if sm.data()[0] != 1 {
        return Err("setup mode is not active".into());
    }

    info!("setup mode is active");
    Ok(())
}

fn find_boot_loader() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let bootcfg = BootConfig::new();

    let Some(current) = bootcfg.current else {
        return Err("BootCurrent variable not present".into());
    };
    debug!("BootCurrent is {current}");

    let Some(entry) = bootcfg.entries.get(&current) else {
        let msg = format!("no boot entry fount for {current}");
        return Err(msg.into());
    };
    debug!("boot entry is \"{}\"", entry.title);

    let Some(efifile) = EfiFile::new_from_dp(&entry.devpath) else {
        return Err("can not find boot loader".into());
    };
    info!("boot loader is \"{}\"", efifile.file.display());

    Ok(efifile.file)
}

fn find_ca_subjects(bootloader: PathBuf) -> Result<Vec<RdnSequence>, Box<dyn std::error::Error>> {
    let binary = std::fs::read(bootloader)?;
    let pefile = pe::PeFile64::parse(binary.as_ref())?;
    let mut subjects = Vec::new();

    let Some(iter) = AttributeCertificateIterator::new(&pefile).expect("enumerate signatures")
    else {
        return Err("no signatures present".into());
    };

    for (i, item) in iter.enumerate() {
        let ac = item.as_ref().expect("get AttributeCertificate");
        let sig = ac
            .get_authenticode_signature()
            .expect("get AuthenticodeSignature");
        let si = sig.signer_info();
        let sid = get_issuer_serial(si)?;
        subjects.push(sid.issuer.clone());

        for (c, cert) in sig.certificates().enumerate() {
            let info = CertInfo::new(cert);
            let desc = info.desc_issuer();
            let good = sid.issuer == cert.tbs_certificate.issuer
                && sid.serial_number == cert.tbs_certificate.serial_number;
            if good {
                info!("  signature {i}, cert {c}, issuer \"{desc}\", ok");
            } else {
                debug!("  signature {i}, cert {c}, issuer \"{desc}\", skip (not issuer)");
            }
        }
    }

    Ok(subjects)
}

#[derive(Debug, Clone)]
struct EnrollCandidateCert {
    guid: &'static Guid,
    cert: &'static [u8],
    prio: u32,
}

impl core::fmt::Display for EnrollCandidateCert {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some(info) = CertInfo::new_from_der(self.cert) {
            write!(f, "{}", info.desc())
        } else {
            write!(f, "invalid certificate")
        }
    }
}

const DB_CERTS: [EnrollCandidateCert; 6] = [
    EnrollCandidateCert {
        guid: &guids::MicrosoftVendor,
        cert: MICROSOFT_DB_UEFI_2011,
        prio: 1,
    },
    EnrollCandidateCert {
        guid: &guids::MicrosoftVendor,
        cert: MICROSOFT_DB_WINDOWS_2011,
        prio: 1,
    },
    EnrollCandidateCert {
        guid: &guids::MicrosoftVendor,
        cert: MICROSOFT_DB_WINDOWS_2023,
        prio: 2,
    },
    EnrollCandidateCert {
        guid: &guids::MicrosoftVendor,
        cert: MICROSOFT_DB_UEFI_2023,
        prio: 2,
    },
    EnrollCandidateCert {
        guid: &guids::MicrosoftVendor,
        cert: MICROSOFT_DB_UEFI_ROM_2023,
        prio: 2,
    },
    EnrollCandidateCert {
        guid: &guids::OvmfEnrollDefaultKeys,
        cert: REDHAT_DB_UEFI_2024,
        prio: 2,
    },
];

fn find_ca_certs(cfg: &Args, subjects: &[RdnSequence]) -> Vec<EnrollCandidateCert> {
    let certs: Vec<_> = DB_CERTS
        .iter()
        .filter_map(|ca| {
            let cert = Certificate::from_der(ca.cert).ok()?;
            if !subjects.contains(&cert.tbs_certificate.subject) {
                return None;
            }
            if let Some(ref subject) = cfg.ca_subject {
                let desc = ca.to_string();
                if !desc.contains(subject) {
                    debug!("  ignoring \"{}\", does not match \"{}\"", desc, subject);
                    return None;
                }
                info!("  using \"{}\", matches \"{}\"", desc, subject);
            }
            Some(ca.clone())
        })
        .collect();

    let maxprio = certs.iter().map(|entry| entry.prio).max().unwrap_or(0);
    certs
        .iter()
        .filter(|entry| entry.prio == maxprio)
        .cloned()
        .collect()
}

fn enroll_one_sigdb(
    cfg: &Args,
    id: ids::ConstVarId,
    sigdb: &EfiSigDB,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("enroll {}", id.name);
    for siglist in &sigdb.siglists {
        if let Some(cert) = siglist.get_x509() {
            if let Some(info) = CertInfo::new_from_der(cert) {
                info!("  x509: \"{}\"", info.desc());
                continue;
            }
        }
        info!("  {}", siglist);
    }

    let ts = sigdb.get_x509_mtime().unwrap_or(EfiTime::dbxupdate());
    let data: Vec<u8> = sigdb.into();
    let auth = esl_to_auth(&ts, &data);
    let var = EfiVar::new_with_vec_full(
        id.into(),
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        auth,
        Some(ts),
    );

    if cfg.dry_run {
        return Ok(());
    }

    sysfs::varstore_write(&var)?;
    Ok(())
}

fn cert_menu(
    certs: &[EnrollCandidateCert],
) -> Result<EnrollCandidateCert, Box<dyn std::error::Error>> {
    let theme = ColorfulTheme::default();
    let sel = Select::with_theme(&theme)
        .with_prompt("select 'db' ca certificate")
        .items(certs)
        .interact_opt()?;

    let Some(s) = sel else {
        return Err("interactive menu canceled".into());
    };
    Ok(certs[s].clone())
}

fn enroll(cfg: &Args) -> Result<(), Box<dyn std::error::Error>> {
    if cfg.dry_run {
        info!("--- dry run mode ---");
    } else {
        check_setup_mode()?;
        check_virtualization()?;
    }

    let mut db = if let Some(profile) = &cfg.profile {
        info!("db: using profile mode ({:?})", profile);
        profile.sigdb()
    } else {
        info!("db: using automatic bootloader mode");
        let loader = find_boot_loader()?;
        let subjects = find_ca_subjects(loader)?;
        let db_certs = find_ca_certs(cfg, &subjects);
        if db_certs.is_empty() {
            if let Some(ref m) = cfg.ca_subject {
                let msg = format!("no 'db' ca certificate matching \"{m}\"");
                return Err(msg.into());
            } else {
                return Err("no 'db' ca certificate".into());
            }
        }

        let enroll = if db_certs.len() > 1 {
            if !cfg.interactive {
                return Err("found multiple signatures, try --ca-subject <match> or -i".into());
            }
            &cert_menu(&db_certs)?
        } else {
            db_certs.first().unwrap()
        };

        let mut sigdb = EfiSigDB::new();
        sigdb.add_x509_from_der(enroll.guid, enroll.cert);
        sigdb
    };

    for filename in &cfg.db_cert {
        let der = cert_loadfile(filename)?;
        db.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, &der);
    }

    let dbx = get_dbx_native();
    enroll_one_sigdb(cfg, ids::DB, &db)?;
    enroll_one_sigdb(cfg, ids::DBX, &dbx)?;
    enroll_one_sigdb(cfg, ids::KEK, &cfg.enroll.kek_sigdb())?;
    enroll_one_sigdb(cfg, ids::PK, &cfg.enroll.sigdb())?;

    if cfg.dry_run {
        info!("dry-run complete");
    } else {
        info!("secure boot setup complete, you can reboot now");
    }
    Ok(())
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    loginit(&cfg);

    match enroll(&cfg) {
        Ok(_) => 0.into(),
        Err(e) => {
            error!("{e}");
            1.into()
        }
    }
}
