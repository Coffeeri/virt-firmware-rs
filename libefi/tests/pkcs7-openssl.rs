#![cfg(feature = "sbdata")]

use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
use openssl::stack::Stack;
use openssl::x509::store::{X509Store, X509StoreBuilder};
use openssl::x509::verify::X509VerifyFlags;
use openssl::x509::{X509PurposeId, X509};

use virtfw_libefi::efivar::auth::EfiVarAuth2;
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;

fn print_cert(msg: &str, cert: &X509) {
    let subject_name = cert.subject_name();
    let issuer_name = cert.issuer_name();
    println!("{msg}");
    println!("  subject: {subject_name:?}");
    println!("  issuer: {issuer_name:?}");
}

// Create a certificate trust store
// Set flags and purpose like edk2 does
fn edk2_store_builder() -> X509StoreBuilder {
    let mut store_builder = X509StoreBuilder::new().unwrap();
    store_builder
        .set_flags(X509VerifyFlags::PARTIAL_CHAIN | X509VerifyFlags::NO_CHECK_TIME)
        .unwrap();
    store_builder.set_purpose(X509PurposeId::ANY).unwrap();
    store_builder
}

fn pk_store() -> X509Store {
    let pk = X509::from_der(REDHAT_PK_KEK_2014).unwrap();
    print_cert("trusted PK cert", &pk);

    let mut store_builder = edk2_store_builder();
    store_builder.add_cert(pk).unwrap();

    store_builder.build()
}

fn kek_store() -> X509Store {
    // trusted kek certificates
    let kek_2011 = X509::from_der(MICROSOFT_KEK_2011).unwrap();
    print_cert("trusted KEK cert", &kek_2011);
    let kek_2023 = X509::from_der(MICROSOFT_KEK_2023).unwrap();
    print_cert("trusted KEK cert", &kek_2023);

    let mut store_builder = edk2_store_builder();
    store_builder.add_cert(kek_2011).unwrap();
    store_builder.add_cert(kek_2023).unwrap();

    store_builder.build()
}

#[test]
fn test_dbx_update_pkcs7() {
    let variable = EfiVar::new_with_vec(
        ids::DBX.into(),
        EfiVarAttr::new_nv_bs_rt()
            .with_time_auth_wr_access(true)
            .with_append_write(true),
        Vec::new(),
    );

    let authvar = EfiVarAuth2::new_from_bytes(DBX_X86_64).expect("Failed to parse DBX update");
    let pkcs7 = Pkcs7::from_der(&authvar.get_pkcs7()).expect("Failed to parse PKCS#7");
    let good_data = authvar.get_signed_data(&variable);
    let bad_data = good_data.get(1..).unwrap();

    // Create (empty) certificate stack + trust stores
    let certs = Stack::new().unwrap();
    let bad_store = pk_store();
    let good_store = kek_store();

    // some logging for trouble shooting
    for signer in pkcs7.signers(&certs, Pkcs7Flags::empty()).unwrap() {
        print_cert("pkcs7 signer cert", &signer);
    }

    // must pass: verify the signature
    let res = pkcs7.verify(
        &certs,
        &good_store,
        Some(&good_data),
        None,
        Pkcs7Flags::BINARY,
    );
    assert!(res.is_ok());

    // must fail: signed data mismatch
    let res = pkcs7.verify(
        &certs,
        &good_store,
        Some(bad_data),
        None,
        Pkcs7Flags::BINARY,
    );
    assert!(res.is_err());

    // must fail: cert not in cert store
    let res = pkcs7.verify(
        &certs,
        &bad_store,
        Some(&good_data),
        None,
        Pkcs7Flags::BINARY,
    );
    assert!(res.is_err());
}
