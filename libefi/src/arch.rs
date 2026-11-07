//! efi architectures

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use clap::{builder::PossibleValue, ValueEnum};

#[derive(Debug, Clone, Copy)]
pub enum EfiArch {
    X64,
    Aa64,
    Riscv64,
}

impl EfiArch {
    pub fn name(self) -> &'static str {
        match self {
            EfiArch::X64 => "x64",
            EfiArch::Aa64 => "aa64",
            EfiArch::Riscv64 => "riscv64",
        }
    }

    pub fn name_uc(self) -> &'static str {
        match self {
            EfiArch::X64 => "X64",
            EfiArch::Aa64 => "AA64",
            EfiArch::Riscv64 => "RISCV64",
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

    #[cfg(target_arch = "riscv64")]
    pub fn native() -> Self {
        EfiArch::Riscv64
    }
}

#[cfg(feature = "std")]
impl ValueEnum for EfiArch {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: [EfiArch; 3] = [
            EfiArch::Aa64,
            EfiArch::X64,
            EfiArch::Riscv64,
        ];
        &VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let pv = match self {
            EfiArch::Aa64 => PossibleValue::new("aa64").aliases(["AA64", "aarch64", "arm64"]),
            EfiArch::X64 => PossibleValue::new("x64").aliases(["X64", "x86_64", "amd64"]),
            EfiArch::Riscv64 => PossibleValue::new("riscv64").aliases(["RISCV64"]),
        };
        Some(pv)
    }
}
