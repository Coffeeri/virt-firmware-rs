use alloc::string::String;
use uguid::Guid;

use virtfw_libefi::efivar::types::EfiVarAttr;

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
