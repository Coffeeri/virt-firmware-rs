//!
//! efi variable store implementation -- handle authenticated variables
//!
use alloc::vec::Vec;
use uguid::Guid;

use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;

use crate::store::EfiVarStore;

impl EfiVarStore {
    fn set_boolean(&mut self, guid: &Guid, name: &str, attr: EfiVarAttr, boolean: bool) {
        let b = if boolean { [1] } else { [0] };
        let var = EfiVar {
            guid: *guid,
            name: name.into(),
            attr,
            data: b.to_vec(),
        };
        self.set_unchecked(var);
    }

    fn set_setup_mode(&mut self, enabled: bool) {
        self.set_boolean(
            &guids::EfiGlobalVariable,
            "SetupMode",
            EfiVarAttr::new()
                .with_bootservice_access(true)
                .with_runtime_access(true),
            enabled,
        );
    }

    fn set_secure_boot(&mut self, enabled: bool) {
        self.set_boolean(
            &guids::EfiGlobalVariable,
            "SecureBoot",
            EfiVarAttr::new()
                .with_bootservice_access(true)
                .with_runtime_access(true),
            enabled,
        );
    }

    fn set_custom_mode(&mut self, enabled: bool) {
        self.set_boolean(
            &guids::EfiCustomModeEnable,
            "CustomMode",
            EfiVarAttr::new()
                .with_non_volatile(true)
                .with_bootservice_access(true),
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
            attr: EfiVarAttr::new()
                .with_bootservice_access(true)
                .with_runtime_access(true),
            data: sigs,
        };
        self.set_unchecked(var);
    }

    fn set_vendor_keys(&mut self) {
        self.set_boolean(
            &guids::EfiGlobalVariable,
            "VendorKeysNv",
            EfiVarAttr::new()
                .with_non_volatile(true)
                .with_bootservice_access(true)
                .with_time_auth_wr_access(true),
            false,
        );
        self.set_boolean(
            &guids::EfiGlobalVariable,
            "VendorKeys",
            EfiVarAttr::new()
                .with_bootservice_access(true)
                .with_runtime_access(true),
            false,
        );
    }

    pub(crate) fn auth_init(&mut self) {
        // TODO: implement secure boot support
        self.set_setup_mode(true);
        self.set_signature_support();
        self.set_secure_boot(false);
        self.set_custom_mode(false);
        self.set_vendor_keys();
    }
}
