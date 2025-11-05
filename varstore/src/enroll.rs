//!
//! efi variable store implementation -- enroll secure boot keys
//!
//use alloc::vec::Vec;

use uguid::Guid;

use virtfw_libefi::efivar::auth::auth_to_esl;
use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::profiles::SecureBootProfile;
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;
use virtfw_libefi::types::EfiTime;

use crate::authvar::{SB_DB, SB_DBX, SB_KEK, SB_PK};
use crate::store::EfiVarStore;

impl EfiVarStore {
    fn enroll_sigdb(&mut self, guid: &Guid, name: &str, ts: Option<EfiTime>, sigdb: &EfiSigDB) {
        let var = EfiVar::new_with_vec_full(
            guid,
            name,
            EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
            sigdb.into(),
            ts,
        );
        self.set_unchecked(var);
    }

    pub fn enroll_pk(&mut self, ts: Option<EfiTime>, pk: &EfiSigDB) {
        self.enroll_sigdb(&guids::EfiGlobalVariable, SB_PK, ts, pk);
    }

    pub fn enroll_kek(&mut self, ts: Option<EfiTime>, kek: &EfiSigDB) {
        self.enroll_sigdb(&guids::EfiGlobalVariable, SB_KEK, ts, kek);
    }

    pub fn enroll_db(&mut self, ts: Option<EfiTime>, db: &EfiSigDB) {
        self.enroll_sigdb(&guids::EfiImageSecurityDatabase, SB_DB, ts, db);
    }

    pub fn enroll_dbx(&mut self, ts: Option<EfiTime>, dbx: &EfiSigDB) {
        self.enroll_sigdb(&guids::EfiImageSecurityDatabase, SB_DBX, ts, dbx);
    }

    pub fn enroll_pk_mgmt(&mut self) {
        let pk = EfiSigDB::new_pk_external_mgmt();
        let ts = EfiTime::initial();
        self.enroll_pk(Some(ts), &pk);
    }

    pub fn enroll_pk_redhat(&mut self) {
        let mut pk = EfiSigDB::new();
        pk.add_x509_from_der(&guids::OvmfEnrollDefaultKeys, REDHAT_PK_KEK_2014);
        let ts = pk.get_x509_mtime();
        self.enroll_pk(ts, &pk);
    }

    pub fn enroll_kek_microsoft(&mut self) {
        let mut kek = EfiSigDB::new();
        kek.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_KEK_2011);
        kek.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_KEK_2023);
        let ts = kek.get_x509_mtime();
        self.enroll_kek(ts, &kek);
    }

    pub fn enroll_db_profile(&mut self, profile: SecureBootProfile) {
        let db = profile.sigdb();
        let ts = db.get_x509_mtime();
        self.enroll_db(ts, &db);
    }

    pub fn enroll_db_microsoft_uefi(&mut self) {
        self.enroll_db_profile(SecureBootProfile::Uefi11);
    }

    // Enroll all certs, including the rh test cert which is used to
    // sign test builds.  The private key is public, so that does not
    // actually protect anything.
    #[cfg(feature = "sbtest")]
    pub fn enroll_db_test_insecure(&mut self) {
        self.enroll_db_profile(SecureBootProfile::RedHatTest);
    }

    pub fn enroll_dbx_native(&mut self) {
        if let Some(auth) = DBX_NATIVE {
            let (ts, esl) = auth_to_esl(auth).unwrap();
            let dbx = EfiSigDB::new_from_bytes(esl).unwrap();
            self.enroll_dbx(Some(ts), &dbx);
        }
    }
}
