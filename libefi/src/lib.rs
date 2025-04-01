#![no_std]

#[cfg(feature = "std")]
pub mod bootcfg;
#[cfg(feature = "std")]
pub mod distro;
#[cfg(feature = "std")]
pub mod efifile;
pub mod efivar;
pub mod guids;
pub mod nocasestr;
pub mod varstore;
