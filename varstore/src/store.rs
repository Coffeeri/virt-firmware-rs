//!
//! efi variable store implementation [WIP]
//!

#![allow(clippy::collapsible_else_if)]
use alloc::vec::Vec;
use uguid::Guid;

use uefi::{Error, Status};
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;

use crate::policy::VariablePolicyEntry;

pub struct EfiVarStore {
    // boot phases
    end_of_dxe: bool,
    ready_to_boot: bool,
    exit_boot_service: bool,

    // efi variables
    variables: Vec<EfiVar>,

    // variable Policies
    pub(crate) policy_disabled: bool,
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

            policy_disabled: false,
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
        self.set_unchecked(new);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.end_of_dxe = false;
        self.ready_to_boot = false;
        self.exit_boot_service = false;

        self.variables.retain(|v| v.attr.non_volatile());
        self.auth_init();

        self.policy_disabled = false;
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

    pub fn auth_init(&mut self) {
        // TODO: implement secure boot support
        self.set_setup_mode(true);
        self.set_signature_support();
        self.set_secure_boot(false);
        self.set_custom_mode(false);
        self.set_vendor_keys();
    }
}

impl Default for EfiVarStore {
    fn default() -> Self {
        Self::new()
    }
}
