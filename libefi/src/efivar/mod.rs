//! efi variable data type support
pub mod auth;
pub mod boot;
pub mod devpath;
#[cfg(feature = "std")]
pub mod io;
pub mod sigdb;
pub mod types;
