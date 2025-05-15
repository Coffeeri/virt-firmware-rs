//! efi guid database

#![allow(non_upper_case_globals)]

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};

use uguid::{guid, Guid};

// efi variable namespaces
pub const EfiCustomModeEnable: Guid = guid!("c076ec0c-7028-4399-a072-71ee5c448b9f");
pub const EfiGlobalVariable: Guid = guid!("8be4df61-93ca-11d2-aa0d-00e098032b8c");
pub const EfiImageSecurityDatabase: Guid = guid!("d719b2cb-3d3a-4596-a3bc-dad00e67656f");
pub const EfiSecureBootEnable: Guid = guid!("f0a30bc7-af08-4556-99c4-001009c93a44");
pub const LoaderInfo: Guid = guid!("4a67b082-0a4c-41cf-b6c7-440b29bb8c4f");
pub const ShimVariable: Guid = guid!("605dab50-e046-4300-abb6-3dd810dd8b23");
pub const VMMBootOrder: Guid = guid!("668f4529-63d0-4bb5-b65d-6fbb9d36a44a");

// efi signatures
pub const EfiCertPkcs7: Guid = guid!("4aafd29d-68df-49ee-8aa9-347d375665a7");
pub const EfiCertSha256: Guid = guid!("c1c41626-504c-4092-aca9-41f936934328");
pub const EfiCertSha384: Guid = guid!("ff3e5307-9fd0-48c9-85f1-8ad56c701e01");
pub const EfiCertSha512: Guid = guid!("093e0fae-a6c4-4f50-9f1b-d41e2b89c19a");
pub const EfiCertRsa2048: Guid = guid!("3c5766e8-269c-4e34-aa14-ed776e85b3b6");
pub const EfiCertX509: Guid = guid!("a5c059a1-94e4-4aa7-87b5-ab155c2bf072");

// efi events
pub const EfiEndOfDxeEventGroup: Guid = guid!("02ce967a-dd7e-4ffc-9ee7-810cf0470880");
pub const EfiEventExitBootServices: Guid = guid!("27abf055-b1b8-4c26-8048-748f37baa2df");
pub const EfiEventReadyToBoot: Guid = guid!("7ce88fb3-4bd7-4679-87a8-a8d8dee50d2b");

// mm protocols
pub const EfiSmmVariableProtocol: Guid = guid!("ed32d533-99e6-4209-9cc0-2d72cdd998a7");
pub const VarCheckPolicyLibMmiHandler: Guid = guid!("da1b0d11-d1a7-46c4-9dc9-f3714875c6eb");

pub fn pretty_name(guid: &Guid) -> String {
    match *guid {
        EfiCustomModeEnable => "EfiCustomModeEnable".to_string(),
        EfiGlobalVariable => "EfiGlobalVariable".to_string(),
        EfiImageSecurityDatabase => "EfiImageSecurityDatabase".to_string(),
        EfiSecureBootEnable => "EfiSecureBootEnable".to_string(),
        LoaderInfo => "LoaderInfo".to_string(),
        ShimVariable => "ShimVariable".to_string(),
        VMMBootOrder => "VMMBootOrder".to_string(),

        EfiCertPkcs7 => "EfiCertPkcs7".to_string(),
        EfiCertSha256 => "EfiCertSha256".to_string(),
        EfiCertSha384 => "EfiCertSha384".to_string(),
        EfiCertSha512 => "EfiCertSha512".to_string(),
        EfiCertRsa2048 => "EfiCertRsa2048".to_string(),
        EfiCertX509 => "EfiCertX509".to_string(),

        EfiEndOfDxeEventGroup => "EfiEndOfDxeEventGroup".to_string(),
        EfiEventExitBootServices => "EfiEventExitBootServices".to_string(),
        EfiEventReadyToBoot => "EfiEventReadyToBoot".to_string(),

        EfiSmmVariableProtocol => "EfiSmmVariableProtocol".to_string(),
        VarCheckPolicyLibMmiHandler => "VarCheckPolicyLibMmiHandler".to_string(),

        _ => format!("{guid}"),
    }
}
