extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::{Deserialize, Serialize};
use uguid::Guid;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonEfiVariable {
    pub guid: Guid,
    pub name: String,
    pub attr: u32,
    pub data: String,
    pub time: Option<String>,
    pub digest: Option<String>,
}

impl fmt::Display for JsonEfiVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} ", self.guid, self.name, self.attr)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonEfiVarStore {
    pub version: u32,
    pub variables: Vec<JsonEfiVariable>,
}
