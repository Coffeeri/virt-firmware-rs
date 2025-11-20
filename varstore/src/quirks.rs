//!
//! efi variable store implementation -- quirks
//!
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::types::EfiVarAttr;

use crate::store::EfiVarStore;

impl EfiVarStore {
    pub fn quirk_disable_shim_reboot(&mut self, enabled: bool) {
        self.set_unchecked_bool(
            ids::FB_NO_REBOOT.into(),
            EfiVarAttr::new_nv_bs_rt(),
            enabled,
        );
    }
}
