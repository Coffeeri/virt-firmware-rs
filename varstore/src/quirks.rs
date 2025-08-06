//!
//! efi variable store implementation -- quirks
//!
use virtfw_libefi::efivar::types::EfiVarAttr;
use virtfw_libefi::guids;

use crate::store::EfiVarStore;

impl EfiVarStore {
    pub fn quirk_disable_shim_reboot(&mut self, enabled: bool) {
        self.set_unchecked_bool(
            &guids::ShimVariable,
            "FB_NO_REBOOT",
            EfiVarAttr::new_nv_bs_rt(),
            enabled,
        );
    }
}
