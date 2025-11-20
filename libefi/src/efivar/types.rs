//! misc ef data types

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Deref;

use bitfield_struct::bitfield;
use uguid::Guid;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::types::EfiTime;

#[bitfield(u32)]
#[derive(FromBytes, IntoBytes, Immutable, KnownLayout, Eq, PartialEq)]
pub struct EfiVarAttr {
    pub non_volatile: bool,
    pub bootservice_access: bool,
    pub runtime_access: bool,
    pub hardware_error: bool,

    pub auth_wr_access: bool, // deprecated
    pub time_auth_wr_access: bool,
    pub append_write: bool,

    #[bits(25)]
    pub reserved: u32,
}

impl EfiVarAttr {
    pub fn new_nv_bs() -> EfiVarAttr {
        EfiVarAttr::new()
            .with_non_volatile(true)
            .with_bootservice_access(true)
    }

    pub fn new_nv_bs_rt() -> EfiVarAttr {
        EfiVarAttr::new()
            .with_non_volatile(true)
            .with_bootservice_access(true)
            .with_runtime_access(true)
    }

    pub fn new_bs_rt() -> EfiVarAttr {
        EfiVarAttr::new()
            .with_bootservice_access(true)
            .with_runtime_access(true)
    }
}

impl core::fmt::Display for EfiVarAttr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut flags = Vec::new();
        if self.non_volatile() {
            flags.push("nv");
        }
        if self.bootservice_access() {
            flags.push("bs");
        }
        if self.runtime_access() {
            flags.push("rt");
        }
        if self.auth_wr_access() || self.time_auth_wr_access() {
            flags.push("auth");
        }
        if self.append_write() {
            flags.push("append");
        }
        write!(f, "[{}]", flags.join(","))
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EfiVarId {
    name: String,
    guid: Guid,
}

impl EfiVarId {
    pub fn new(name: String, guid: Guid) -> Self {
        EfiVarId { name, guid }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn guid(&self) -> &Guid {
        &self.guid
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EfiVar {
    id: EfiVarId,
    attr: EfiVarAttr,
    data: Vec<u8>,
    time: Option<EfiTime>,
}

impl EfiVar {
    pub fn new_with_vec(id: EfiVarId, attr: EfiVarAttr, data: Vec<u8>) -> Self {
        let time = if attr.time_auth_wr_access() {
            Some(EfiTime::default())
        } else {
            None
        };
        Self {
            id,
            attr,
            data,
            time,
        }
    }

    pub fn new_with_vec_full(
        id: EfiVarId,
        attr: EfiVarAttr,
        data: Vec<u8>,
        time: Option<EfiTime>,
    ) -> Self {
        Self {
            id,
            attr,
            data,
            time,
        }
    }

    pub fn new_from_slice(id: EfiVarId, attr: EfiVarAttr, data: &[u8]) -> Self {
        Self::new_with_vec(id, attr, data.into())
    }

    pub fn new_from_bool(id: EfiVarId, attr: EfiVarAttr, value: bool) -> Self {
        let data = if value { vec![1] } else { vec![0] };
        Self::new_with_vec(id, attr, data)
    }

    pub fn attr(&self) -> EfiVarAttr {
        self.attr
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn time(&self) -> Option<&EfiTime> {
        self.time.as_ref()
    }
}

impl Deref for EfiVar {
    type Target = EfiVarId;

    fn deref(&self) -> &EfiVarId {
        &self.id
    }
}
