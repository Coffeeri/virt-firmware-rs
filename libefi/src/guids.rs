//! efi guid database

#![allow(non_upper_case_globals)]

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};

use uguid::{guid, Guid};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

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
pub const EfiCertExternalMgmt: Guid = guid!("452e8ced-dfff-4b8c-ae01-5118862e682c");

// efi signature owners
pub const MicrosoftVendor: Guid = guid!("77fa9abd-0359-4d32-bd60-28f4e78f784b");
pub const OvmfEnrollDefaultKeys: Guid = guid!("a0baa8a3-041d-48a8-bc87-c36d121b5e3d");

// efi events
pub const EfiEndOfDxeEventGroup: Guid = guid!("02ce967a-dd7e-4ffc-9ee7-810cf0470880");
pub const EfiEventExitBootServices: Guid = guid!("27abf055-b1b8-4c26-8048-748f37baa2df");
pub const EfiEventReadyToBoot: Guid = guid!("7ce88fb3-4bd7-4679-87a8-a8d8dee50d2b");

// mm protocols
pub const EfiSmmVariableProtocol: Guid = guid!("ed32d533-99e6-4209-9cc0-2d72cdd998a7");
pub const VarCheckPolicyLibMmiHandler: Guid = guid!("da1b0d11-d1a7-46c4-9dc9-f3714875c6eb");

// config tables
pub const Acpi2Cfg: Guid = guid!("8868e871-e4f1-11d3-bc22-0080c73c8881");
pub const AcpiCfg: Guid = guid!("eb9d2d30-2d88-11d3-9a16-0090273fc14d");
pub const DebugImageInfoCfg: Guid = guid!("49152e77-1ada-4764-b7a2-7afefed95e8b");
pub const DxeServicesCfg: Guid = guid!("05ad34ba-6f02-4214-952e-4da0398e2bb9");
pub const HandOffBlockListCfg: Guid = guid!("7739f24c-93d7-11d4-9a3a-0090273fc14d");
pub const MemoryTypeInfoCfg: Guid = guid!("4c19049f-4137-4dd3-9c10-8b97a83ffdfa");
pub const SmbiosCfg: Guid = guid!("eb9d2d31-2d88-11d3-9a16-0090273fc14d");

// guid HOBs
pub const FdtHob: Guid = guid!("16958446-19B7-480B-B047-7485AD3F716D");
pub const OvmfPkgPlatformInfo: Guid = guid!("dec9b486-1f16-47c7-8f68-df1a41888ba5");
pub const PcdDatabase: Guid = guid!("9b3ada4f-ae56-4c24-8dea-f03b7558ae50");
pub const SecPlatformInformation2: Guid = guid!("9e9f374b-8f16-4230-9824-5846ee766a97");
pub const OvmfMemDebugLog: Guid = guid!("95305139-b20f-4723-8425-627c888ff121");

pub fn try_pretty_name(guid: &Guid) -> Option<&'static str> {
    match *guid {
        EfiCustomModeEnable => Some("EfiCustomModeEnable"),
        EfiGlobalVariable => Some("EfiGlobalVariable"),
        EfiImageSecurityDatabase => Some("EfiImageSecurityDatabase"),
        EfiSecureBootEnable => Some("EfiSecureBootEnable"),
        LoaderInfo => Some("LoaderInfo"),
        ShimVariable => Some("ShimVariable"),
        VMMBootOrder => Some("VMMBootOrder"),

        EfiCertPkcs7 => Some("EfiCertPkcs7"),
        EfiCertSha256 => Some("EfiCertSha256"),
        EfiCertSha384 => Some("EfiCertSha384"),
        EfiCertSha512 => Some("EfiCertSha512"),
        EfiCertRsa2048 => Some("EfiCertRsa2048"),
        EfiCertX509 => Some("EfiCertX509"),
        EfiCertExternalMgmt => Some("EfiCertExternalMgmt"),

        MicrosoftVendor => Some("MicrosoftVendor"),
        OvmfEnrollDefaultKeys => Some("OvmfEnrollDefaultKeys"),

        EfiEndOfDxeEventGroup => Some("EfiEndOfDxeEventGroup"),
        EfiEventExitBootServices => Some("EfiEventExitBootServices"),
        EfiEventReadyToBoot => Some("EfiEventReadyToBoot"),

        EfiSmmVariableProtocol => Some("EfiSmmVariableProtocol"),
        VarCheckPolicyLibMmiHandler => Some("VarCheckPolicyLibMmiHandler"),

        Acpi2Cfg => Some("Acpi2Cfg"),
        AcpiCfg => Some("AcpiCfg"),
        DebugImageInfoCfg => Some("DebugImageInfoCfg"),
        DxeServicesCfg => Some("DxeServicesCfg"),
        HandOffBlockListCfg => Some("HandOffBlockListCfg"),
        MemoryTypeInfoCfg => Some("MemoryTypeInfoCfg"),
        SmbiosCfg => Some("SmbiosCfg"),

        FdtHob => Some("FdtHob"),
        OvmfPkgPlatformInfo => Some("OvmfPkgPlatformInfo"),
        PcdDatabase => Some("PcdDatabase"),
        SecPlatformInformation2 => Some("SecPlatformInformation2"),

        _ => None,
    }
}

pub fn pretty_name(guid: &Guid) -> String {
    match try_pretty_name(guid) {
        Some(name) => name.to_string(),
        None => format!("{guid}"),
    }
}

// workaround for uguid::Guid not having zerocopy support
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct RawGuid {
    bytes: [u8; 16],
}

impl From<RawGuid> for Guid {
    fn from(raw: RawGuid) -> Self {
        Self::from_bytes(raw.bytes)
    }
}

impl From<Guid> for RawGuid {
    fn from(guid: Guid) -> Self {
        Self {
            bytes: guid.to_bytes(),
        }
    }
}
