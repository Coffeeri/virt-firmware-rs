use clap::Parser;
use virtfw_libefi::sb::certs::profiles::{PkEnroll, SecureBootProfile};

#[derive(Parser, Debug)]
#[command(
    version,
    name = "enroll-sb-vars",
    about = "enroll databases and enable secure boot",
    long_about = r#"Enroll databases and enable secure boot.

Enroll secure boot certificates.  By default the utility will look at
your boot loader (usually shim.efi) and enroll the CA certificate used
to sign the boot loader.  In case the boot loader has multiple
signatures you might have to pick one using '--ca-subject'.
Alternatively you can pick a set of certificates using '--profile'.
Additional certificates can be loaded from files using '--db-cert'.
You can use dry run mode to verify the utility will do what you want
before actually enrolling the certificates.

This needs a recent UEFI firmware version which accepts an unsigned
platform key (PK).  The config switch (PcdRequireSelfSignedPk) was
added to edk2 in 2023.  The default for OVMF has been changed in
upstream edk2 in 2026 (edk2-stable202605 or newer).  When using the
qemu variable store (aka '-device uefi-vars-x64') you need qemu 11.2
or newer.

This utility is restricted to virtual machines.

"#
)]
pub struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    pub loglevel: log::Level,

    /// dry run mode
    ///
    /// List the certificates which would be enrolled without actually
    /// updating the UEFI secure boot configuration.
    #[arg(long)]
    pub dry_run: bool,

    /// select 'PK' enroll mode
    #[arg(long, value_name = "MODE",
          value_enum, default_value_t = PkEnroll::RedHat)]
    pub enroll: PkEnroll,

    /// enable interactive 'db' certificate selection
    #[arg(long, short)]
    pub interactive: bool,

    /// select 'db' boot loader certificate
    ///
    /// Might be needed if your boot loader has multiple signatures to
    /// specify which signature CA should be used.
    #[arg(long, value_name = "CN")]
    pub ca_subject: Option<String>,

    /// select 'db' certificate profile
    ///
    /// For linux systems you usually need 'uefi11' or 'uefi23'.
    #[arg(long, value_name = "NAME")]
    pub profile: Option<SecureBootProfile>,

    /// add certificate to 'db'
    ///
    /// Can be specified multiple times.  Certificates will be
    /// enrolled in addition to the other certificates.  Use
    /// '--profile none' if you want specify all certificates
    /// explicitly.
    #[arg(long, value_name = "FILE")]
    pub db_cert: Vec<String>,

    #[arg(long, hide = true)]
    pub manpage: bool,
}
