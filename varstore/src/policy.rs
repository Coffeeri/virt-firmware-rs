//!
//! efi variable store implementation -- variable policies
//!
use alloc::string::String;
use uguid::Guid;

use virtfw_libefi::efivar::types::EfiVarAttr;

use crate::store::EfiVarStore;

#[derive(Debug)]
pub enum VariablePolicyType {
    NoLock,
    LockNow,
    LockOnCreate,
    LockOnVarState {
        namespace: Guid,
        name: String,
        value: u8,
    },
}

#[derive(Debug)]
pub struct VariablePolicyEntry {
    pub namespace: Guid,
    pub name: String,
    pub min_size: u32,
    pub max_size: u32,
    pub attr_must_have: EfiVarAttr,
    pub attr_cant_have: EfiVarAttr,
    pub lock_policy_type: VariablePolicyType,
}

impl core::fmt::Display for VariablePolicyEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "policy")?;
        write!(f, " name={}", self.name)?;
        write!(f, " size=0x{:x}-0x{:x}", self.min_size, self.max_size)?;
        write!(f, " must={}", self.attr_must_have)?;
        write!(f, " cant={}", self.attr_cant_have)?;
        write!(f, " type={:?}", self.lock_policy_type)?;
        Ok(())
    }
}

impl EfiVarStore {
    pub fn policy_disable(&mut self) {
        self.policy_disabled = true
    }

    pub fn policy_is_enabled(&self) -> bool {
        !self.policy_disabled
    }

    pub fn policy_register(&mut self, entry: VariablePolicyEntry) {
        // FIXME: check for existing entry
        // FIXME: check for lock
        self.policies.push(entry);
    }

    pub fn policy_lock(&mut self) {
        self.policy_locked = true
    }
}
