//! misc ef data types

use uuid::Uuid;

#[derive(Debug)]
pub struct EfiVarId {
    pub name: String,
    pub guid: Uuid,
}

#[derive(Debug)]
pub struct EfiVar {
    pub name: String,
    pub guid: Uuid,
    pub attr: u32,
    pub data: Vec<u8>,
}
