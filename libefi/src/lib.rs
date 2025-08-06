#![doc = include_str!("../README.md")]
#![no_std]
#[cfg(feature = "std")]
pub mod bootcfg;
#[cfg(feature = "std")]
pub mod distro;
#[cfg(feature = "udev")]
pub mod efifile;
pub mod efivar;
pub mod guids;
pub mod nocasestr;
#[cfg(feature = "sbdata")]
pub mod sb;
pub mod types;
pub mod varstore;
