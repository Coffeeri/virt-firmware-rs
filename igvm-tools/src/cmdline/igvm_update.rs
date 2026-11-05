use clap::{Parser, ValueHint};
use virtfw_libefi::arch::EfiArch;
use virtfw_libefi::sb::certs::profiles::{PkEnroll, SecureBootProfile};

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-update",
          about = "add kernel to igvm file",
          long_about = None)]
pub struct Args {
    /// input igvm image
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub input: String,

    /// input shim image (optional)
    #[arg(short, long, value_name = "SHIM", value_hint = ValueHint::FilePath)]
    pub shim: Option<String>,

    /// input kernel image (can be any efi image)
    #[arg(short, long, value_name = "KERNEL", value_hint = ValueHint::FilePath)]
    pub kernel: Option<String>,

    /// add authenticode sha256 hashes for shim and kernel to 'db'
    #[cfg(feature = "authenticode")]
    #[arg(long, visible_alias = "add-hash")]
    pub add_hash_sha256: bool,

    /// add microsoft certificates to 'db'
    #[arg(long)]
    pub add_cert_ms: bool,

    /// select 'db' profile
    #[arg(long, value_name = "NAME")]
    pub profile: Option<SecureBootProfile>,

    /// select 'PK' enroll mode
    #[arg(long, value_name = "MODE",
          value_enum, default_value_t = PkEnroll::RedHat)]
    pub enroll: PkEnroll,

    /// select efi architecture for 'dbx'
    #[arg(long, value_name = "ARCH")]
    pub arch: Option<EfiArch>,

    /// inspect updated igvm
    #[arg(long)]
    pub inspect: bool,

    /// output igvm image
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub output: Option<String>,
}
