//!
//! efi variable store implementation -- handle authenticated variables
//!
use alloc::vec::Vec;

use uefi::{Error, Status};

use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;

use crate::store::EfiVarStore;

// signature database names
pub const SB_PK: &str = "PK";
pub const SB_KEK: &str = "KEK";
pub const SB_DB: &str = "db";
pub const SB_DBX: &str = "dbx";

// secure boot variable names
pub const SB_SETUP_MODE: &str = "SetupMode";
pub const SB_CUSTOM_MODE: &str = "CustomMode";
pub const SB_SECURE_BOOT: &str = "SecureBoot";

impl EfiVarStore {
    fn set_setup_mode(&mut self, enabled: bool) {
        self.set_unchecked_bool(
            &guids::EfiGlobalVariable,
            SB_SETUP_MODE,
            EfiVarAttr::new_bs_rt(),
            enabled,
        );
    }

    fn set_secure_boot(&mut self, enabled: bool) {
        self.set_unchecked_bool(
            &guids::EfiGlobalVariable,
            SB_SECURE_BOOT,
            EfiVarAttr::new_bs_rt(),
            enabled,
        );
    }

    fn set_custom_mode(&mut self, enabled: bool) {
        self.set_unchecked_bool(
            &guids::EfiCustomModeEnable,
            SB_CUSTOM_MODE,
            EfiVarAttr::new_nv_bs(),
            enabled,
        );
    }

    fn set_signature_support(&mut self) {
        let mut sigs = Vec::new();
        sigs.extend_from_slice(&guids::EfiCertSha256.to_bytes());
        sigs.extend_from_slice(&guids::EfiCertSha384.to_bytes());
        sigs.extend_from_slice(&guids::EfiCertSha512.to_bytes());
        sigs.extend_from_slice(&guids::EfiCertRsa2048.to_bytes());
        sigs.extend_from_slice(&guids::EfiCertX509.to_bytes());
        let var = EfiVar {
            guid: guids::EfiGlobalVariable,
            name: "SignatureSupport".into(),
            attr: EfiVarAttr::new_bs_rt(),
            data: sigs,
        };
        self.set_unchecked(var);
    }

    fn set_vendor_keys(&mut self) {
        self.set_unchecked_bool(
            &guids::EfiGlobalVariable,
            "VendorKeysNv",
            EfiVarAttr::new_nv_bs().with_time_auth_wr_access(true),
            false,
        );
        self.set_unchecked_bool(
            &guids::EfiGlobalVariable,
            "VendorKeys",
            EfiVarAttr::new_bs_rt(),
            false,
        );
    }

    fn check_time_auth(&self, _variable: &EfiVar) -> Result<(), Error<&'static str>> {
        // not implemented yet
        Err(Error::new(Status::UNSUPPORTED, "time auth bit set"))
    }

    pub(crate) fn check_auth(&self, variable: &EfiVar) -> Result<(), Error<&'static str>> {
        if variable.attr.auth_wr_access() {
            // not used any more
            return Err(Error::new(Status::UNSUPPORTED, "auth bit set"));
        }
        if variable.attr.time_auth_wr_access() {
            return self.check_time_auth(variable);
        }
        Ok(())
    }

    pub(crate) fn auth_init(&mut self) {
        let setup = self
            .get_unchecked(SB_PK, &guids::EfiGlobalVariable)
            .is_none();
        self.set_setup_mode(setup);

        self.set_signature_support();

        // TODO: SecureBootEnable

        let sb = !setup;
        self.set_secure_boot(sb);

        self.set_custom_mode(false);
        self.set_vendor_keys();
    }
}
