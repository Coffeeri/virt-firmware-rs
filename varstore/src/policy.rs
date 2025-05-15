//!
//! efi variable store implementation -- variable policies
//!
use alloc::string::String;
use uguid::Guid;

use uefi::{Error, Status};
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
    fn policy_find(&self, name: &str, guid: &Guid) -> Option<&VariablePolicyEntry> {
        self.policies
            .iter()
            .find(|p| p.name == name && p.namespace == *guid)
    }

    fn policies_sort(&mut self) {
        // sort by count of '#' wildcard chars
        // less wildcards (more specific matches) go first
        self.policies.sort_by(|a, b| {
            let aw = a.name.chars().filter(|c| *c == '#').count();
            let bw = b.name.chars().filter(|c| *c == '#').count();
            aw.cmp(&bw)
        });
    }

    fn policies_dump(&self) {
        log::debug!("---");
        for p in self.policies.iter() {
            log::debug!("  {p}");
        }
        log::debug!("---");
    }

    pub fn policy_disable(&mut self) {
        self.policy_disabled = true
    }

    pub fn policy_is_enabled(&self) -> bool {
        !self.policy_disabled
    }

    pub fn policy_register(
        &mut self,
        entry: VariablePolicyEntry,
    ) -> Result<(), Error<&'static str>> {
        if self.policy_locked {
            return Err(Error::new(Status::WRITE_PROTECTED, "policies locked"));
        }
        if self.policy_find(&entry.name, &entry.namespace).is_some() {
            return Err(Error::new(Status::ALREADY_STARTED, "duplicate entry"));
        }
        self.policies.push(entry);
        self.policies_sort();
        self.policies_dump();
        Ok(())
    }

    pub fn policy_lock(&mut self) {
        self.policy_locked = true
    }
}
