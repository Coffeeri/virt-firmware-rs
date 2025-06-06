//!
//! efi variable store implementation -- enroll secure boot keys
//!
//use alloc::vec::Vec;

use uguid::Guid;

use virtfw_libefi::efivar::sigdb::EfiSigDB;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;
use virtfw_libefi::sb::certs::*;
use virtfw_libefi::sb::dbx::*;

use crate::store::EfiVarStore;

impl EfiVarStore {
    fn enroll_sigdb(&mut self, guid: &Guid, name: &str, sigdb: &EfiSigDB) {
        let var = EfiVar {
            guid: *guid,
            name: name.into(),
            attr: EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
            data: sigdb.into(),
        };
        self.set_unchecked(var);
    }

    fn enroll_pk(&mut self, pk: &EfiSigDB) {
        self.enroll_sigdb(&guids::EfiGlobalVariable, "PK", pk);
    }

    fn enroll_db(&mut self, db: &EfiSigDB) {
        self.enroll_sigdb(&guids::EfiImageSecurityDatabase, "db", db);
    }

    fn enroll_dbx(&mut self, dbx: &EfiSigDB) {
        self.enroll_sigdb(&guids::EfiImageSecurityDatabase, "dbx", dbx);
    }

    pub fn enroll_pk_mgmt(&mut self) {
        let pk = EfiSigDB::new_pk_external_mgmt();
        self.enroll_pk(&pk);
    }

    pub fn enroll_db_microsoft_uefi(&mut self) {
        let mut db = EfiSigDB::new();
        db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
        db.add_x509_from_der(&guids::MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
        self.enroll_db(&db);
    }

    pub fn enroll_dbx_native(&mut self) {
        if let Some(auth) = DBX_NATIVE {
            let esl = auth_to_esl(auth).unwrap();
            let dbx = EfiSigDB::new_from_bytes(esl).unwrap();
            self.enroll_dbx(&dbx);
        }
    }
}
