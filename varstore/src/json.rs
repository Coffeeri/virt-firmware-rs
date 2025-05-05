//!
//! read/write json efi varstore
//!
//! https://gitlab.com/qemu-project/qemu/-/blob/master/qapi/uefi.json
//!
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::From;
use core::fmt;
use core::ops::Deref;

use serde::{Deserialize, Serialize};
use uguid::Guid;

use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonHexString(String);

impl Deref for JsonHexString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<JsonHexString> for Vec<u8> {
    type Error = hex::FromHexError;
    fn try_from(hs: JsonHexString) -> Result<Self, Self::Error> {
        hex::decode(hs.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonEfiVariable {
    pub guid: Guid,
    pub name: String,
    pub attr: u32,
    pub data: JsonHexString,
    pub time: Option<JsonHexString>,
    pub digest: Option<JsonHexString>,
}

impl TryFrom<JsonEfiVariable> for EfiVar {
    type Error = hex::FromHexError;
    fn try_from(v: JsonEfiVariable) -> Result<Self, Self::Error> {
        let attr = EfiVarAttr::from(v.attr);
        let data = Vec::<u8>::try_from(v.data)?;
        Ok(EfiVar {
            guid: v.guid,
            name: v.name,
            attr,
            data,
        })
    }
}

impl fmt::Display for JsonEfiVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let attr = EfiVarAttr::from(self.attr);
        write!(
            f,
            "{:36} {:24} {:3} byte(s)  {}",
            guids::pretty_name(&self.guid),
            self.name,
            self.data.len() / 2,
            attr,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonEfiVarStore {
    pub version: u32,
    pub variables: Vec<JsonEfiVariable>,
}
