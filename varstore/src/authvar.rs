//!
//! efi variable store implementation -- handle authenticated variables
//!
use alloc::vec::Vec;
use uefi::{Error, Status};

use virtfw_libefi::efivar::auth::EfiVarAuth2;
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
pub const SB_SECURE_BOOT_ENABLE: &str = "SecureBootEnable";

impl EfiVarStore {
    fn get_setup_mode(&self) -> bool {
        self.get_unchecked(SB_PK, &guids::EfiGlobalVariable)
            .is_none()
    }

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

    fn get_secure_boot_enable(&mut self) -> Option<bool> {
        let sbe = self.get_unchecked(SB_SECURE_BOOT_ENABLE, &guids::EfiSecureBootEnable)?;
        Some(sbe.data[0] != 0)
    }

    pub fn set_secure_boot_enable(&mut self, enabled: bool) {
        self.set_unchecked_bool(
            &guids::EfiSecureBootEnable,
            SB_SECURE_BOOT_ENABLE,
            EfiVarAttr::new_nv_bs(),
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
        let var = EfiVar::new_with_vec(
            &guids::EfiGlobalVariable,
            "SignatureSupport",
            EfiVarAttr::new_bs_rt(),
            sigs,
        );
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

    fn is_sb_pk(&self, var: &EfiVar) -> bool {
        var.guid == guids::EfiGlobalVariable && var.name() == SB_PK
    }

    fn is_sb_kek(&self, var: &EfiVar) -> bool {
        var.guid == guids::EfiGlobalVariable && var.name() == SB_KEK
    }

    // image security database (db + dbx)
    fn is_sb_imgdb(&self, var: &EfiVar) -> bool {
        var.guid == guids::EfiImageSecurityDatabase && (var.name() == SB_DB || var.name() == SB_DBX)
    }

    // is secure boot auth var?
    fn is_sb_authvar(&self, var: &EfiVar) -> bool {
        self.is_sb_pk(var) || self.is_sb_kek(var) || self.is_sb_imgdb(var)
    }

    // check secure boot auth vars
    fn check_time_auth_sb(
        &self,
        _variable: &EfiVar,
        _authvar: &EfiVarAuth2,
    ) -> Result<(), Error<&'static str>> {
        if self.get_setup_mode() {
            return Ok(());
        }

        // TODO: pkcs7 signature verification -> make vars read-only for now
        Err(Error::new(Status::WRITE_PROTECTED, "secure boot auth var"))
    }

    fn check_time_auth(
        &self,
        variable: &EfiVar,
        authvar: &EfiVarAuth2,
    ) -> Result<(), Error<&'static str>> {
        if self.is_sb_authvar(variable) {
            return self.check_time_auth_sb(variable, authvar);
        }

        // TODO: pkcs7 signature verification + store certificate hash
        Err(Error::new(Status::UNSUPPORTED, "time auth bit set"))
    }

    pub(crate) fn set_time_auth(&mut self, variable: &EfiVar) -> Result<(), Error<&'static str>> {
        let Some(authvar) = EfiVarAuth2::new_from_bytes(&variable.data) else {
            return Err(Error::new(Status::INVALID_PARAMETER, "authvar parse error"));
        };

        self.check_time_auth(variable, &authvar)?;

        let v =
            EfiVar::new_from_slice(&variable.guid, variable.name(), variable.attr, authvar.data);

        if v.data.is_empty() {
            self.delete_unchecked(v);
        } else {
            self.set_unchecked(v);
        }

        if self.is_sb_pk(variable) {
            self.auth_init();
        }

        Ok(())
    }

    pub(crate) fn auth_init(&mut self) {
        let setup = self.get_setup_mode();
        self.set_setup_mode(setup);

        self.set_signature_support();

        let sb;
        if let Some(sbe) = self.get_secure_boot_enable() {
            sb = !setup && sbe;
        } else {
            sb = !setup;
        }

        self.set_secure_boot(sb);

        self.set_custom_mode(false);
        self.set_vendor_keys();
    }
}
