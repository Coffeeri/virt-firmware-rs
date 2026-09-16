#[cfg(feature = "x509")]
pub mod info;
#[cfg(all(feature = "x509", feature = "std"))]
pub mod loadfile;
#[cfg(feature = "x509")]
pub mod oids;
pub mod profiles;

pub const REDHAT_PK_KEK_2014: &[u8] = include_bytes!("redhat.com/rh-pk-kek-2014.der");
pub const MICROSOFT_PK_2023: &[u8] = include_bytes!("microsoft.com/ms-oem-pk-2023.der");
pub const MICROSOFT_KEK_2011: &[u8] = include_bytes!("microsoft.com/ms-kek-2011.der");
pub const MICROSOFT_KEK_2023: &[u8] = include_bytes!("microsoft.com/ms-kek-2023.der");

pub const REDHAT_DB_UEFI_2024: &[u8] = include_bytes!("redhat.com/rh-uefi-2024.der");
pub const MICROSOFT_DB_UEFI_2011: &[u8] = include_bytes!("microsoft.com/ms-uefi-2011.der");
pub const MICROSOFT_DB_UEFI_2023: &[u8] = include_bytes!("microsoft.com/ms-uefi-2023.der");
pub const MICROSOFT_DB_UEFI_ROM_2023: &[u8] = include_bytes!("microsoft.com/ms-uefi-rom-2023.der");

pub const MICROSOFT_DB_WINDOWS_2011: &[u8] = include_bytes!("microsoft.com/windows-2011.der");
pub const MICROSOFT_DB_WINDOWS_2023: &[u8] = include_bytes!("microsoft.com/windows-2023.der");

// used for test builds, private key is public
#[cfg(feature = "sbtest")]
pub const REDHAT_DB_TEST_INSECURE_2012: &[u8] =
    include_bytes!("redhat.com/redhat-test-insecure-2012.der");
