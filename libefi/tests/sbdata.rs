#![cfg(feature = "sbdata")]

use virtfw_libefi::efivar::auth::auth_to_esl;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;

const PK_REDHAT: &[u8] = include_bytes!("data/pk_redhat.esl");
const DB_MICROSOFT: &[u8] = include_bytes!("data/db_microsoft.esl");

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
fn parse_dbxupdate_aarch64() {
    let esl = auth_to_esl(DBX_AARCH64).unwrap();
    let sigdb = EfiSigDB::new_from_bytes(esl).unwrap();
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 26);
}

#[test]
fn parse_dbxupdate_x86_64() {
    let esl = auth_to_esl(DBX_X86_64).unwrap();
    let sigdb = EfiSigDB::new_from_bytes(esl).unwrap();
    let siglist = sigdb.siglists.first().unwrap();
    assert_eq!(siglist.entries.len(), 430);
}
