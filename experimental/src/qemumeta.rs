use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FirmwareOsInterface {
    BIOS,
    OpenFirmware,
    SVSM,
    #[allow(non_camel_case_types)]
    uboot,
    UEFI,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum FirmwareArchitecture {
    aarch64,
    arm,
    i386,
    loongarch64,
    riscv64,
    x86_64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareTarget {
    pub architecture: FirmwareArchitecture,
    pub machines: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum FirmwareFormat {
    raw,
    qcow2,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum FirmwareFlashMode {
    split,
    combined,
    stateless,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareFlashFile {
    pub filename: String,
    pub format: FirmwareFormat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareMappingFlash {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<FirmwareFlashMode>,
    pub executable: FirmwareFlashFile,
    #[serde(skip_serializing_if = "Option::is_none", rename = "nvram-template")]
    pub nvram_template: Option<FirmwareFlashFile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareMappingKernel {
    pub filename: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareMemoryUefiVars {
    pub template: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareMappingMemory {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "uefi-vars")]
    pub uefi_vars: Option<FirmwareMemoryUefiVars>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirmwareMappingIgvm {
    pub filename: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "device")]
pub enum FirmwareMapping {
    Flash(FirmwareMappingFlash),
    Igvm(FirmwareMappingIgvm),
    Kernel(FirmwareMappingKernel),
    Memory(FirmwareMappingMemory),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FirmwareFeature {
    #[serde(rename = "acpi-s3")]
    AcpiS3,
    #[serde(rename = "acpi-s4")]
    AcpiS4,
    #[serde(rename = "amd-sev")]
    AmdSev,
    #[serde(rename = "amd-sev-es")]
    AmdSevEs,
    #[serde(rename = "amd-sev-snp")]
    AmdSevSnp,
    #[serde(rename = "enrolled-keys")]
    EnrolledKeys,
    #[serde(rename = "host-uefi-vars")]
    HostUefiVars,
    #[serde(rename = "intel-tdx")]
    IntelTdx,
    #[serde(rename = "requires-smm")]
    RequiresSmm,
    #[serde(rename = "secure-boot")]
    SecureBoot,
    #[serde(rename = "verbose-dynamic")]
    VerboseDynamic,
    #[serde(rename = "verbose-static")]
    VerboseStatic,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QemuFirmware {
    pub description: String,
    #[serde(rename = "interface-types")]
    pub interface_types: Vec<FirmwareOsInterface>,
    pub mapping: FirmwareMapping,
    pub targets: Vec<FirmwareTarget>,
    pub features: Vec<FirmwareFeature>,
    pub tags: Vec<String>,
}
