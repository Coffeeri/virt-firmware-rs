//!
//! efi variable store implementation [WIP]
//!

#![allow(clippy::collapsible_else_if)]
use alloc::vec::Vec;
use uguid::Guid;

use uefi::{Error, Status};
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};

use crate::policy::{VariablePolicyEntry, VariablePolicyType};

pub struct EfiVarStore {
    // boot phases
    pub(crate) end_of_dxe: bool,
    ready_to_boot: bool,
    exit_boot_service: bool,

    // efi variables
    variables: Vec<EfiVar>,

    // variable Policies
    pub(crate) policy_locked: bool,
    pub(crate) policies: Vec<VariablePolicyEntry>,
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
        }
    }

    fn get_position(&self, name: &str, guid: &Guid) -> Option<usize> {
        self.variables
            .iter()
            .position(|v| v.name == name && v.guid == *guid)
    }

    fn get_unchecked(&self, name: &str, guid: &Guid) -> Option<&EfiVar> {
        let index = self.get_position(name, guid)?;
        Some(&self.variables[index])
    }

    pub(crate) fn set_unchecked(&mut self, variable: EfiVar) {
        let index = self.get_position(&variable.name, &variable.guid);
        if let Some(i) = index {
            // replace
            if false {
                self.variables[i] = variable;
            } else {
                // qemu compatible ordering
                self.variables.remove(i);
                self.variables.push(variable);
            }
        } else {
            // append
            self.variables.push(variable);
        }
    }

    pub(crate) fn set_unchecked_bool(
        &mut self,
        guid: &Guid,
        name: &str,
        attr: EfiVarAttr,
        boolean: bool,
    ) {
        let b = if boolean { [1] } else { [0] };
        let var = EfiVar {
            guid: *guid,
            name: name.into(),
            attr,
            data: b.to_vec(),
        };
        self.set_unchecked(var);
    }

    pub fn check_old_new(old: &EfiVar, new: &EfiVar) -> Result<(), Error<&'static str>> {
        if old.attr != new.attr {
            return Err(Error::new(Status::INVALID_PARAMETER, "attribute mismatch"));
        }
        Ok(())
    }

    pub fn check_service(&self, variable: &EfiVar) -> Result<(), Error<&'static str>> {
        if !self.exit_boot_service {
            // boot service
            if !variable.attr.bootservice_access() {
                return Err(Error::new(Status::NOT_FOUND, "BS attr not set"));
            }
        } else {
            // runtime
            if !variable.attr.runtime_access() {
                return Err(Error::new(Status::NOT_FOUND, "RT attr not set"));
            }
        };
        Ok(())
    }

    pub fn check_attributes(&self, variable: &EfiVar) -> Result<(), Error<&'static str>> {
        if variable.attr.reserved() != 0 {
            return Err(Error::new(Status::INVALID_PARAMETER, "reserved bit set"));
        }
        if variable.attr.hardware_error() {
            return Err(Error::new(Status::INVALID_PARAMETER, "hw-error bit set"));
        }
        if variable.attr.auth_wr_access() {
            // not used any more
            return Err(Error::new(Status::UNSUPPORTED, "auth bit set"));
        }
        if variable.attr.time_auth_wr_access() {
            // not implemented yet
            return Err(Error::new(Status::UNSUPPORTED, "time auth bit set"));
        }
        Ok(())
    }

    pub fn get(&self, name: &str, guid: &Guid) -> Result<&EfiVar, Error<&'static str>> {
        if let Some(var) = self.get_unchecked(name, guid) {
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

    pub fn get_next(&self, name: &str, guid: &Guid) -> Result<(&str, &Guid), Error<&'static str>> {
        let mut next_index = 0;
        if !name.is_empty() {
            let Some(index) = self.get_position(name, guid) else {
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
            return Ok((&var.name, &var.guid));
        }
    }

    pub fn delete(&mut self, new: EfiVar) -> Result<(), Error<&'static str>> {
        let old_opt = self.get_unchecked(&new.name, &new.guid);
        if let Some(old) = old_opt {
            self.check_service(old)?;
            self.policy_check(&new, false)?;
            self.variables
                .retain(|v| v.name != new.name || v.guid != new.guid);
        }
        Ok(())
    }

    pub fn set(&mut self, new: EfiVar) -> Result<(), Error<&'static str>> {
        if new.data.is_empty() {
            return self.delete(new);
        }
        let old_opt = self.get_unchecked(&new.name, &new.guid);
        if let Some(old) = old_opt {
            Self::check_old_new(old, &new)?;
        }
        self.check_service(&new)?;
        self.check_attributes(&new)?;

        let is_new_var = old_opt.is_none();
        self.policy_check(&new, is_new_var)?;

        self.set_unchecked(new);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.end_of_dxe = false;
        self.ready_to_boot = false;
        self.exit_boot_service = false;

        self.variables.retain(|v| v.attr.non_volatile());
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
