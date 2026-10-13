use clap::Parser;
use virtfw_libefi::arch::EfiArch;
use virtfw_libefi::sb::certs::profiles::SecureBootProfile;

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-update",
          about = "add kernel to igvm file",
          long_about = None)]
pub struct Args {
    /// input igvm image
    #[arg(short, long, value_name = "FILE", required_unless_present = "manpage")]
    pub input: Option<String>,

    /// input shim image (optional)
    #[arg(short, long, value_name = "SHIM")]
    pub shim: Option<String>,

    /// input kernel image (can be any efi image)
    #[arg(short, long, value_name = "KERNEL")]
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

    /// select efi architecture for 'dbx'
    #[arg(long, value_name = "ARCH")]
    pub arch: Option<EfiArch>,

    /// inspect updated igvm
    #[arg(long)]
    pub inspect: bool,

    /// output igvm image
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    #[arg(long, hide = true)]
    pub manpage: bool,
}
