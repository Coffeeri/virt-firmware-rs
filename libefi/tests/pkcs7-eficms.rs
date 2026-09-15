#![cfg(feature = "eficms")]

use virtfw_libefi::eficms::openssl::EfiCmsOpenSSL;
use virtfw_libefi::eficms::{EfiCms, EfiCmsCert};
use virtfw_libefi::efivar::auth::EfiVarAuth2;
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;

#[test]
fn test_dbx_update_pkcs7() {
    let variable = EfiVar::new_with_vec(
        ids::DBX.into(),
        EfiVarAttr::new_nv_bs_rt()
            .with_time_auth_wr_access(true)
            .with_append_write(true),
        Vec::new(),
    );

    let authvar = EfiVarAuth2::new_from_bytes(DBX_2011_X86_64).expect("Failed to parse DBX update");
    let signature = authvar.get_pkcs7();
    let good_data = authvar.get_signed_data(&variable);
    let bad_data = good_data.get(1..).unwrap();

    let pk = vec![REDHAT_PK_KEK_2014.to_vec()];
    let kek = vec![MICROSOFT_KEK_2011.to_vec(), MICROSOFT_KEK_2011.to_vec()];

    let eficms: &mut dyn EfiCms = &mut EfiCmsOpenSSL::new();
    eficms.add_certs_vec(EfiCmsCert::PK, pk);
    eficms.add_certs_vec(EfiCmsCert::KEK, kek);

    // must pass: verify the signature
    let res = eficms.verify_signature(&signature, &good_data);
    assert!(res.is_ok());

    // must pass: verify the certificate is KEK
    let res = eficms.verify_certificate(&signature, &good_data).unwrap();
    assert_eq!(res, EfiCmsCert::KEK);

    // must fail: signed data mismatch
    let res = eficms.verify_signature(&signature, &bad_data);
    assert!(res.is_err());
}
