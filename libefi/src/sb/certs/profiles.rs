extern crate alloc;
use alloc::format;
use alloc::string::String;
use core::str::FromStr;

use crate::efivar::sigdb::EfiSigDB;
use crate::guids::MicrosoftVendor;
use crate::sb::certs::*;

#[derive(Debug, Clone)]
pub enum SecureBootProfile {
    // no certificates
    None,

    // microsoft profiles
    Win11,
    Win23,
    Uefi11,
    Uefi23,
    All,
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
            SecureBootProfile::Uefi11 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
            }
            SecureBootProfile::Uefi23 => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
            }
            SecureBootProfile::All => {
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_WINDOWS_2023);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2011);
                sigdb.add_x509_from_der(&MicrosoftVendor, MICROSOFT_DB_UEFI_2023);
            }
        }
        sigdb
    }
}

impl FromStr for SecureBootProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "win11" => Ok(Self::Win11),
            "win23" => Ok(Self::Win23),
            "uefi11" => Ok(Self::Uefi11),
            "uefi23" => Ok(Self::Uefi23),
            "all" => Ok(Self::All),
            _ => Err(format!("unknown profile: {s}")),
        }
    }
}
