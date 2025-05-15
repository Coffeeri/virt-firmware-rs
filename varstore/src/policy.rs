//!
//! efi variable store implementation -- variable policies
//!
use alloc::string::String;
use uguid::Guid;

use uefi::{Error, Status};
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};

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
    fn match_wildcard(pe: &VariablePolicyEntry, var: &EfiVar) -> bool {
        if pe.namespace != var.guid {
            return false;
        }
        if pe.name.len() != var.name.len() {
            return false;
        }
        pe.name
            .chars()
            .zip(var.name.chars())
            .all(|(p, v)| p == '#' || p == v)
    }

    fn policy_find_wildcard(&self, var: &EfiVar) -> Option<&VariablePolicyEntry> {
        self.policies.iter().find(|p| Self::match_wildcard(p, var))
    }

    pub fn policy_check(&self, var: &EfiVar, is_new_var: bool) -> Result<(), Error<&'static str>> {
        if !self.end_of_dxe {
            return Ok(());
        };

        let Some(policy) = self.policy_find_wildcard(var) else {
            return Ok(());
        };

        if var.data.len() < policy.min_size as usize {
            return Err(Error::new(Status::INVALID_PARAMETER, "policy min-size"));
        }
        if var.data.len() > policy.max_size as usize {
            return Err(Error::new(Status::INVALID_PARAMETER, "policy max-size"));
        }

        let attr = u32::from(var.attr);
        let must = u32::from(policy.attr_must_have);
        let cant = u32::from(policy.attr_cant_have);
        if (attr & must) != must {
            return Err(Error::new(Status::INVALID_PARAMETER, "policy attr must"));
        }
        if (attr & cant) != 0 {
            return Err(Error::new(Status::INVALID_PARAMETER, "policy attr cant"));
        }

        match &policy.lock_policy_type {
            VariablePolicyType::NoLock => {}
            VariablePolicyType::LockNow => {
                return Err(Error::new(Status::WRITE_PROTECTED, "policy lock-now"));
            }
            VariablePolicyType::LockOnCreate => {
                if !is_new_var {
                    return Err(Error::new(Status::WRITE_PROTECTED, "policy lock-on-create"));
                }
            }
            VariablePolicyType::LockOnVarState {
                namespace,
                name,
                value,
            } => {
                if let Ok(lockvar) = self.get(name, namespace) {
                    if lockvar.data.len() == 1 && lockvar.data[0] == *value {
                        return Err(Error::new(
                            Status::WRITE_PROTECTED,
                            "policy lock-on-var-state",
                        ));
                    }
                }
            }
        }

        Ok(())
    }

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
