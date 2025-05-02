extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use uguid::Guid;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonEfiVariable {
    pub guid: Guid,
    pub name: String,
    pub attr: u32,
    pub data: String,
    pub time: Option<String>,
    pub digest: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonVarStore {
    pub version: u32,
    pub variables: Vec<JsonEfiVariable>,
}
