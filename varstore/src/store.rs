#![allow(clippy::collapsible_else_if)]

extern crate alloc;
use alloc::vec::Vec;
use uguid::Guid;

use uefi::{Error, Status};
use virtfw_libefi::efivar::types::EfiVar;

pub struct EfiVarStore {
    exit_boot_service: bool,
    variables: Vec<EfiVar>,
}

impl EfiVarStore {
    pub fn new() -> Self {
        let variables = Vec::new();
        let exit_boot_service = false;
        Self {
            variables,
            exit_boot_service,
        }
    }

    fn get_unchecked(&self, name: &str, guid: &Guid) -> Option<&EfiVar> {
        self.variables
            .iter()
            .find(|&v| v.name == name && v.guid == *guid)
    }

    fn set_unchecked(&mut self, variable: EfiVar) {
        let pos = self
            .variables
            .iter()
            .position(|v| v.name == variable.name && v.guid == variable.guid);
        if let Some(index) = pos {
            // replace
            self.variables[index] = variable;
        } else {
            // append
            self.variables.push(variable);
        }
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

    pub fn get(&self, name: &str, guid: &Guid) -> Result<&EfiVar, Error<&'static str>> {
        if let Some(var) = self.get_unchecked(name, guid) {
            self.check_service(var)?;
            return Ok(var);
        }
        Err(Error::new(Status::NOT_FOUND, "variable not found"))
    }

    pub fn set(&mut self, new: EfiVar) -> Result<(), Error<&'static str>> {
        let old_opt = self.get_unchecked(&new.name, &new.guid);
        if let Some(old) = old_opt {
            Self::check_old_new(old, &new)?;
        }
        self.check_service(&new)?;
        self.set_unchecked(new);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.exit_boot_service = false;
        self.variables.retain(|v| v.attr.non_volatile());
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
