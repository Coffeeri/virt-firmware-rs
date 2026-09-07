#![cfg(feature = "sbdata")]

use virtfw_libefi::efivar::auth::auth_to_esl;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;

const PK_REDHAT: &[u8] = include_bytes!("data/pk_redhat.esl");
const DB_MICROSOFT: &[u8] = include_bytes!("data/db_microsoft.esl");

fn log_init() {
    let _ = env_logger::builder()
        .filter_module(module_path!(), log::LevelFilter::Debug)
        .filter_module("virtfw_libefi", log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

#[test]
fn enroll_pk_redhat() {
    let mut pk = EfiSigDB::new();
    pk.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, REDHAT_PK_KEK_2014);
    let bytes: Vec<u8> = (&pk).into();
    assert_eq!(PK_REDHAT, bytes)
}

#[test]
fn enroll_db_microsoft() {
    let mut db = EfiSigDB::new();
    db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
    db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
    let bytes: Vec<u8> = (&db).into();
    assert_eq!(DB_MICROSOFT, bytes)
}

#[test]
fn parse_dbxupdate_2011_aarch64() {
    let (_ts, esl) = auth_to_esl(DBX_2011_AARCH64).unwrap();
    let sigdb = EfiSigDB::new_from_bytes(esl).unwrap();
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 26);
}

#[test]
fn parse_dbxupdate_2011_x86_64() {
    let (_ts, esl) = auth_to_esl(DBX_2011_X86_64).unwrap();
    let sigdb = EfiSigDB::new_from_bytes(esl).unwrap();
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 443);
}

#[test]
fn parse_dbxupdate_2023_aarch64() {
    let (_ts, esl) = auth_to_esl(DBX_2023_AARCH64).unwrap();
    let sigdb = EfiSigDB::new_from_bytes(esl).unwrap();
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 22);
}

#[test]
fn parse_dbxupdate_2023_x86_64() {
    let (_ts, esl) = auth_to_esl(DBX_2023_X86_64).unwrap();
    let sigdb = EfiSigDB::new_from_bytes(esl).unwrap();
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 291);
}

#[test]
fn parse_dbxupdate_merge_x86_64() {
    log_init();
    let (_ts, esl11) = auth_to_esl(DBX_2011_X86_64).unwrap();
    let (_ts, esl23) = auth_to_esl(DBX_2023_X86_64).unwrap();
    let sigdb11 = EfiSigDB::new_from_bytes(esl11).unwrap();
    let sigdb23 = EfiSigDB::new_from_bytes(esl23).unwrap();
    let mut sigdb = EfiSigDB::new();
    sigdb.merge(&sigdb11);
    sigdb.merge(&sigdb23);
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 445);
}

#[test]
fn parse_dbxupdate_merge_aarch64() {
    log_init();
    let (_ts, esl11) = auth_to_esl(DBX_2011_AARCH64).unwrap();
    let (_ts, esl23) = auth_to_esl(DBX_2023_AARCH64).unwrap();
    let sigdb11 = EfiSigDB::new_from_bytes(esl11).unwrap();
    let sigdb23 = EfiSigDB::new_from_bytes(esl23).unwrap();
    let mut sigdb = EfiSigDB::new();
    sigdb.merge(&sigdb11);
    sigdb.merge(&sigdb23);
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 26);
}
