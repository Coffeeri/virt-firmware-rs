//! efi guid database

#![allow(non_upper_case_globals)]

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};

use uguid::{guid, Guid};

// efi variable namespaces
pub const EfiGlobalVariable: Guid = guid!("8be4df61-93ca-11d2-aa0d-00e098032b8c");
pub const LoaderInfo: Guid = guid!("4a67b082-0a4c-41cf-b6c7-440b29bb8c4f");
pub const VMMBootOrder: Guid = guid!("668f4529-63d0-4bb5-b65d-6fbb9d36a44a");

// mm protocols
pub const EfiSmmVariableProtocol: Guid = guid!("ed32d533-99e6-4209-9cc0-2d72cdd998a7");
pub const VarCheckPolicyLibMmiHandler: Guid = guid!("da1b0d11-d1a7-46c4-9dc9-f3714875c6eb");

pub fn pretty_name(guid: &Guid) -> String {
    match *guid {
        EfiGlobalVariable => "EfiGlobalVariable".to_string(),
        LoaderInfo => "LoaderInfo".to_string(),
        VMMBootOrder => "VMMBootOrder".to_string(),

        EfiSmmVariableProtocol => "EfiSmmVariableProtocol".to_string(),
        VarCheckPolicyLibMmiHandler => "VarCheckPolicyLibMmiHandler".to_string(),

        _ => format!("{guid}"),
    }
}
