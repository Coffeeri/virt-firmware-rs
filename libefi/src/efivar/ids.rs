use uguid::Guid;

use crate::efivar::types::EfiVarId;
use crate::guids;

pub struct ConstVarId {
    pub name: &'static str,
    pub guid: &'static Guid,
}

impl ConstVarId {
    pub const fn new(name: &'static str, guid: &'static Guid) -> Self {
        Self { name, guid }
    }
}

impl From<ConstVarId> for EfiVarId {
    fn from(c: ConstVarId) -> Self {
        Self::new(c.name.into(), *c.guid)
    }
}

impl PartialEq<ConstVarId> for &EfiVarId {
    fn eq(&self, other: &ConstVarId) -> bool {
        self.name() == other.name && self.guid() == other.guid
    }
}

pub const BOOT_NEXT: ConstVarId = ConstVarId::new("BootNext", &guids::EfiGlobalVariable);
pub const BOOT_ORDER: ConstVarId = ConstVarId::new("BootOrder", &guids::EfiGlobalVariable);

pub const PK: ConstVarId = ConstVarId::new("PK", &guids::EfiGlobalVariable);
pub const KEK: ConstVarId = ConstVarId::new("KEK", &guids::EfiGlobalVariable);
pub const DB: ConstVarId = ConstVarId::new("db", &guids::EfiImageSecurityDatabase);
pub const DBX: ConstVarId = ConstVarId::new("dbx", &guids::EfiImageSecurityDatabase);

pub const SETUP_MODE: ConstVarId = ConstVarId::new("SetupMode", &guids::EfiGlobalVariable);
pub const SECURE_BOOT: ConstVarId = ConstVarId::new("SecureBoot", &guids::EfiGlobalVariable);
pub const CUSTOM_MODE: ConstVarId = ConstVarId::new("CustomMode", &guids::EfiCustomModeEnable);
pub const SECURE_BOOT_ENABLE: ConstVarId =
    ConstVarId::new("SecureBootEnable", &guids::EfiSecureBootEnable);

pub const SIGNATURE_SUPPORT: ConstVarId =
    ConstVarId::new("SignatureSupport", &guids::EfiGlobalVariable);
pub const VENDOR_KEYS: ConstVarId = ConstVarId::new("VendorKeys", &guids::EfiGlobalVariable);
pub const VENDOR_KEYS_NV: ConstVarId = ConstVarId::new("VendorKeysNv", &guids::EfiGlobalVariable);

pub const FB_NO_REBOOT: ConstVarId = ConstVarId::new("FB_NO_REBOOT", &guids::ShimVariable);
pub const FALLBACK_VERBOSE: ConstVarId = ConstVarId::new("FALLBACK_VERBOSE", &guids::ShimVariable);
pub const SHIM_VERBOSE: ConstVarId = ConstVarId::new("SHIM_VERBOSE", &guids::ShimVariable);
