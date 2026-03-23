//!
//! efi variable store implementation -- core
//!

#![allow(clippy::collapsible_else_if)]
use alloc::vec::Vec;
use core::ops::Deref;
use uguid::Guid;

use uefi::{Error, Status};
use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr, EfiVarId};

use crate::policy::{VariablePolicyEntry, VariablePolicyType};

#[derive(Default)]
pub struct EfiVarStoreConfig {
    pub allow_custom_mode: bool,
    pub enforce_secure_boot: bool,
}

impl EfiVarStoreConfig {
    pub const fn new() -> Self {
        Self {
            allow_custom_mode: false,
            enforce_secure_boot: false,
        }
    }

    pub const fn new_sb() -> Self {
        Self {
            allow_custom_mode: false,
            enforce_secure_boot: true,
        }
    }
}

pub(crate) struct VariableEntry {
    pub(crate) var: EfiVar,
    pub(crate) inode: Option<u32>,
    pub(crate) modified: bool,
}

impl VariableEntry {
    pub(crate) fn new(var: EfiVar, inode: Option<u32>, modified: bool) -> Self {
        Self {
            var,
            inode,
            modified,
        }
    }
}

impl Deref for VariableEntry {
    type Target = EfiVar;

    fn deref(&self) -> &EfiVar {
        &self.var
    }
}

#[derive(Clone)]
pub struct EfiVarStoreChanges {
    pub variable_added: bool,
    pub variable_updated: bool,
    pub variable_removed: bool,
}

impl EfiVarStoreChanges {
    const fn new() -> Self {
        Self {
            variable_added: false,
            variable_updated: false,
            variable_removed: false,
        }
    }

    pub fn is_unchanged(&self) -> bool {
        !self.variable_added && !self.variable_updated && !self.variable_removed
    }

    pub(crate) fn add(&mut self) {
        self.variable_added = true;
    }

    pub(crate) fn update(&mut self) {
        self.variable_updated = true;
    }

    pub(crate) fn remove(&mut self) {
        self.variable_removed = true;
    }

    pub(crate) fn clear(&mut self) {
        self.variable_added = false;
        self.variable_updated = false;
        self.variable_removed = false;
    }
}

pub struct EfiVarStore {
    // boot phases
    pub(crate) end_of_dxe: bool,
    ready_to_boot: bool,
    exit_boot_service: bool,

    // efi variables
    pub(crate) variables: Vec<VariableEntry>,

    // variable Policies
    pub(crate) policy_locked: bool,
    pub(crate) policies: Vec<VariablePolicyEntry>,

    // persistance
    pub(crate) modified: EfiVarStoreChanges,

    // configuration
    pub(crate) config: EfiVarStoreConfig,
}

impl EfiVarStore {
    pub const fn new() -> Self {
        let variables = Vec::new();
        let policies = Vec::new();
        Self {
            end_of_dxe: false,
            ready_to_boot: false,
            exit_boot_service: false,

            variables,

            policy_locked: false,
            policies,

            modified: EfiVarStoreChanges::new(),
            config: EfiVarStoreConfig::new(),
        }
    }

    pub fn configure(&mut self, config: EfiVarStoreConfig) {
        self.config = config
    }

    pub(crate) fn get_position(&self, id: &EfiVarId) -> Option<usize> {
        self.variables.iter().position(|v| v.deref().deref() == id)
    }

    pub(crate) fn get_unchecked(&self, id: &EfiVarId) -> Option<&EfiVar> {
        let index = self.get_position(id)?;
        Some(&self.variables[index])
    }

    pub(crate) fn set_unchecked(&mut self, variable: EfiVar) {
        let nv = variable.attr().non_volatile();
        let index = self.get_position(variable.deref());
        if let Some(i) = index {
            // replace
            if false {
                self.variables[i].var = variable;
                self.variables[i].modified = nv;
            } else {
                // qemu compatible ordering
                let inode = self.variables[i].inode;
                let entry = VariableEntry::new(variable, inode, nv);
                self.variables.remove(i);
                self.variables.push(entry);
            }
            if nv {
                self.modified.update()
            }
        } else {
            // append
            let entry = VariableEntry::new(variable, None, nv);
            self.variables.push(entry);
            if nv {
                self.modified.add()
            }
        }
    }

    pub(crate) fn set_unchecked_bool(&mut self, id: EfiVarId, attr: EfiVarAttr, boolean: bool) {
        let var = EfiVar::new_from_bool(id, attr, boolean);
        self.set_unchecked(var);
    }

    pub(crate) fn delete_unchecked(&mut self, new: EfiVar) {
        let nv = new.attr().non_volatile();
        self.variables
            .retain(|v| v.name() != new.name() || v.guid() != new.guid());
        if nv {
            self.modified.remove()
        }
    }

    pub fn check_old_new(&self, old: &EfiVar, new: &EfiVar) -> Result<(), Error<&'static str>> {
        if old.attr() != new.attr() {
            return Err(Error::new(Status::INVALID_PARAMETER, "attribute mismatch"));
        }
        Ok(())
    }

    pub fn check_service(&self, variable: &EfiVar) -> Result<(), Error<&'static str>> {
        if !self.exit_boot_service {
            // boot service
            if !variable.attr().bootservice_access() {
                return Err(Error::new(Status::NOT_FOUND, "BS attr not set"));
            }
        } else {
            // runtime
            if !variable.attr().runtime_access() {
                return Err(Error::new(Status::NOT_FOUND, "RT attr not set"));
            }
        };
        Ok(())
    }

    pub fn check_attributes(&self, variable: &EfiVar) -> Result<(), Error<&'static str>> {
        if variable.attr().reserved() != 0 {
            return Err(Error::new(Status::INVALID_PARAMETER, "reserved bit set"));
        }
        if variable.attr().hardware_error() {
            return Err(Error::new(Status::INVALID_PARAMETER, "hw-error bit set"));
        }
        if variable.attr().auth_wr_access() {
            // not used any more
            return Err(Error::new(Status::UNSUPPORTED, "auth bit set"));
        }
        Ok(())
    }

    // extra checks for secure boot variables
    pub fn check_secure_boot(&self, variable: &EfiVar) -> Result<(), Error<&'static str>> {
        if variable.deref() == ids::SETUP_MODE {
            return Err(Error::new(
                Status::WRITE_PROTECTED,
                "SetupMode is read-only",
            ));
        }

        if variable.deref() == ids::SECURE_BOOT_ENABLE && self.config.enforce_secure_boot {
            return Err(Error::new(
                Status::WRITE_PROTECTED,
                "secure boot must be active",
            ));
        }

        if variable.deref() == ids::CUSTOM_MODE && !self.config.allow_custom_mode {
            return Err(Error::new(
                Status::WRITE_PROTECTED,
                "custom mode is disabled",
            ));
        }

        Ok(())
    }

    pub fn get(&self, id: &EfiVarId) -> Result<&EfiVar, Error<&'static str>> {
        if let Some(var) = self.get_unchecked(id) {
            self.check_service(var)?;
            return Ok(var);
        }
        Err(Error::new(Status::NOT_FOUND, "variable not found"))
    }

    pub fn lock(&mut self, name: &str, guid: &Guid) -> Result<(), Error<&'static str>> {
        let policy = VariablePolicyEntry {
            name: name.into(),
            namespace: *guid,
            min_size: 0,
            max_size: u32::MAX,
            attr_must_have: 0.into(),
            attr_cant_have: 0.into(),
            lock_policy_type: VariablePolicyType::LockNow,
        };
        self.policy_register(policy)?;
        Ok(())
    }

    pub fn get_next(&self, id: &EfiVarId) -> Result<&EfiVarId, Error<&'static str>> {
        let mut next_index = 0;
        if !id.name().is_empty() {
            let Some(index) = self.get_position(id) else {
                return Err(Error::new(Status::NOT_FOUND, "variable not found"));
            };
            let var = self.variables.get(index).unwrap();
            self.check_service(var)?;
            next_index = index + 1;
        }

        loop {
            let var_opt = self.variables.get(next_index);
            if var_opt.is_none() {
                return Err(Error::new(Status::NOT_FOUND, "end of list"));
            }
            let var = var_opt.unwrap();
            if self.check_service(var).is_err() {
                next_index += 1;
                continue;
            }
            return Ok(var);
        }
    }

    pub fn delete(&mut self, new: EfiVar) -> Result<(), Error<&'static str>> {
        let old_opt = self.get_unchecked(&new);
        if let Some(old) = old_opt {
            self.check_service(old)?;
            self.check_attributes(old)?;
            self.check_secure_boot(old)?;
            self.policy_check(&new, false)?;
            if old.attr().time_auth_wr_access() {
                return Err(Error::new(
                    Status::SECURITY_VIOLATION,
                    "tried to delete auth var without pkcs7 signature",
                ));
            }
            self.delete_unchecked(new);
        }
        Ok(())
    }

    pub fn set(&mut self, new: EfiVar) -> Result<(), Error<&'static str>> {
        if new.data().is_empty() {
            return self.delete(new);
        }
        let old_opt = self.get_unchecked(&new);
        if let Some(old) = old_opt {
            self.check_old_new(old, &new)?;
        }
        self.check_service(&new)?;
        self.check_attributes(&new)?;
        self.check_secure_boot(&new)?;
        self.policy_check(&new, old_opt.is_none())?;

        if new.attr().time_auth_wr_access() {
            return self.set_time_auth(&new);
        }

        self.set_unchecked(new);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.end_of_dxe = false;
        self.ready_to_boot = false;
        self.exit_boot_service = false;

        self.variables.retain(|v| v.attr().non_volatile());
        self.auth_init();

        self.policy_locked = false;
        self.policies.clear();
    }

    pub fn end_of_dxe(&mut self) {
        self.end_of_dxe = true;
    }

    pub fn ready_to_boot(&mut self) {
        self.ready_to_boot = true;
    }

    pub fn exit_boot_service(&mut self) {
        self.exit_boot_service = true;
    }
}

impl Default for EfiVarStore {
    fn default() -> Self {
        Self::new()
    }
}
