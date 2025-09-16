#![cfg(feature = "sbdata")]

use virtfw_libefi::efivar::auth::{auth_to_esl, esl_to_auth};
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::types::EfiTime;

#[test]
fn authvar() {
    let mut pk = EfiSigDB::new();
    pk.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, REDHAT_PK_KEK_2014);
    let ts = EfiTime::now();
    let esl = Vec::<u8>::from(&pk);
    let auth = esl_to_auth(&ts, &esl);
    let verify = auth_to_esl(&auth).unwrap();
    assert_eq!(esl, verify);
}
