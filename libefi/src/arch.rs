//! efi architectures

extern crate alloc;
use alloc::format;
use alloc::string::String;
use core::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum EfiArch {
    X64,
    Aa64,
}

impl EfiArch {
    pub fn name(self) -> &'static str {
        match self {
            EfiArch::X64 => "x64",
            EfiArch::Aa64 => "aa64",
        }
    }

    pub fn name_uc(self) -> &'static str {
        match self {
            EfiArch::X64 => "X64",
            EfiArch::Aa64 => "AA64",
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn native() -> Self {
        EfiArch::Aa64
    }

    #[cfg(target_arch = "x86_64")]
    pub fn native() -> Self {
        EfiArch::X64
    }
}

impl FromStr for EfiArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "x64" | "x86_64" | "amd64" => Ok(Self::X64),
            "aa64" | "aarch64" | "arm64" => Ok(Self::Aa64),
            _ => Err(format!("unknown arch: {s}")),
        }
    }
}
