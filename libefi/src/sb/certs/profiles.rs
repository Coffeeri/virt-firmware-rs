#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use clap::{builder::PossibleValue, ValueEnum};

use crate::efivar::sigdb::EfiSigDB;
use crate::guids::MicrosoftVendor;
#[cfg(feature = "sbtest")]
use crate::guids::OvmfEnrollDefaultKeys;
use crate::sb::certs::*;

#[derive(Debug, Clone)]
pub enum SecureBootProfile {
    // no certificates
    None,

    // microsoft profiles
    Win11,
    Win23,
    WinRom23,
    Uefi11,
    Uefi23,
    UefiRom23,
    All,

    // Enroll all certs, including the rh test cert which is used to
    // sign test builds.  The private key is public, so that does not
    // actually protect anything.
    #[cfg(feature = "sbtest")]
    RedHatTest,
}

impl SecureBootProfile {
    pub fn sigdb(&self) -> EfiSigDB {
        let mut sigdb = EfiSigDB::new();
        match self {
            SecureBootProfile::None => {}

            SecureBootProfile::Win11 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
            }
            SecureBootProfile::Win23 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
            }
            SecureBootProfile::WinRom23 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_ROM_2023);
            }

            SecureBootProfile::Uefi11 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
            }
            SecureBootProfile::Uefi23 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
            }
            SecureBootProfile::UefiRom23 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_ROM_2023);
            }

            SecureBootProfile::All => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
            }

            #[cfg(feature = "sbtest")]
            SecureBootProfile::RedHatTest => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
                sigdb.add_x509_from_der(&OvmfEnrollDefaultKeys, REDHAT_DB_UEFI_2024);
                sigdb.add_x509_from_der(&OvmfEnrollDefaultKeys, REDHAT_DB_TEST_INSECURE_2012);
            }
        }
        sigdb
    }
}

#[cfg(feature = "std")]
impl ValueEnum for SecureBootProfile {
    fn value_variants<'a>() -> &'a [Self] {
        #[cfg(feature = "sbtest")]
        const SIZE: usize = 9;
        #[cfg(not(feature = "sbtest"))]
        const SIZE: usize = 8;
        static VARIANTS: [SecureBootProfile; SIZE] = [
            SecureBootProfile::None,
            SecureBootProfile::Win11,
            SecureBootProfile::Win23,
            SecureBootProfile::WinRom23,
            SecureBootProfile::Uefi11,
            SecureBootProfile::Uefi23,
            SecureBootProfile::UefiRom23,
            SecureBootProfile::All,
            #[cfg(feature = "sbtest")]
            SecureBootProfile::RedHatTest,
        ];
        &VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let pv = match self {
            SecureBootProfile::None => PossibleValue::new("none").help("no certificates"),
            SecureBootProfile::Win11 => {
                PossibleValue::new("win11").help("Windows CA certificates, 2011 + 2023")
            }
            SecureBootProfile::Win23 => {
                PossibleValue::new("win23").help("Windows CA certificate, 2023 only")
            }
            SecureBootProfile::WinRom23 => PossibleValue::new("win+rom23")
                .help("Windows CA + Option ROM certificates, 2023 only"),
            SecureBootProfile::Uefi11 => {
                PossibleValue::new("uefi11").help("UEFI CA certificates, 2011 + 2023")
            }
            SecureBootProfile::Uefi23 => {
                PossibleValue::new("uefi23").help("UEFI CA certificate, 2023")
            }
            SecureBootProfile::UefiRom23 => {
                PossibleValue::new("uefi+rom23").help("UEFI CA + Option ROM certificates, 2023")
            }
            SecureBootProfile::All => {
                PossibleValue::new("all").help("Both Windows and UEFI CA certificates, 2011 + 2023")
            }
            #[cfg(feature = "sbtest")]
            SecureBootProfile::RedHatTest => PossibleValue::new("rh-test"),
        };
        Some(pv)
    }
}
