//!
//! read/write json efi varstore
//!
//! <https://gitlab.com/qemu-project/qemu/-/blob/master/qapi/uefi.json>
//!
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::From;
use core::fmt;
use core::ops::Deref;

use serde::{Deserialize, Serialize};
use uguid::Guid;
use zerocopy::{FromBytes, IntoBytes};

use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;
use virtfw_libefi::types::EfiTime;

use crate::store::EfiVarStore;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct JsonHexString(String);

impl Deref for JsonHexString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&JsonHexString> for Vec<u8> {
    type Error = hex::FromHexError;
    fn try_from(hs: &JsonHexString) -> Result<Self, Self::Error> {
        hex::decode(&hs.0)
    }
}

impl From<&[u8]> for JsonHexString {
    fn from(data: &[u8]) -> Self {
        Self(hex::encode(data))
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct JsonEfiVariable {
    pub guid: Guid,
    pub name: String,
    pub attr: u32,
    pub data: JsonHexString,
    pub time: Option<JsonHexString>,
    pub digest: Option<JsonHexString>,
}

impl JsonEfiVariable {
    pub fn parse_time(&self) -> Option<EfiTime> {
        let hexstr = self.time.as_ref()?;
        let bytes = Vec::<u8>::try_from(hexstr).ok()?;
        let efitime = EfiTime::read_from_bytes(&bytes).ok()?;
        Some(efitime)
    }
}

impl TryFrom<&JsonEfiVariable> for EfiVar {
    type Error = hex::FromHexError;
    fn try_from(v: &JsonEfiVariable) -> Result<Self, Self::Error> {
        let attr = EfiVarAttr::from(v.attr);
        let data = Vec::<u8>::try_from(&v.data)?;
        let time = v.parse_time();
        Ok(EfiVar::new_with_vec_full(
            &v.guid, &v.name, attr, data, time,
        ))
    }
}

impl From<&EfiVar> for JsonEfiVariable {
    fn from(v: &EfiVar) -> Self {
        Self {
            guid: *v.guid(),
            name: v.name().into(),
            attr: v.attr().into(),
            data: JsonHexString::from(v.data()),
            time: v.time.map(|t| JsonHexString::from(t.as_bytes())),
            digest: None,
        }
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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct JsonEfiVarStore {
    pub version: u32,
    pub variables: Vec<JsonEfiVariable>,
}

impl EfiVarStore {
    pub fn json_import(&mut self, jstore: &JsonEfiVarStore) {
        for jvar in &jstore.variables {
            let var = EfiVar::try_from(jvar).unwrap();
            self.set_unchecked(var);
        }
    }

    pub fn json_export(&self) -> JsonEfiVarStore {
        let variables = self
            .variables
            .iter()
            .filter(|v| v.attr().non_volatile())
            .map(JsonEfiVariable::from)
            .collect();
        JsonEfiVarStore {
            version: 2,
            variables,
        }
    }
}
