use clap::{Parser, ValueEnum};
use virtfw_libefi::sb::certs::profiles::{PkEnroll, SecureBootProfile};

#[derive(Debug, Clone, ValueEnum)]
pub enum EfiCmsImpl {
    #[value(name = "null", help = "dummy implementation")]
    Null,
    #[value(name = "openssl", help = "OpenSSL implementation (default)")]
    OpenSSLPkcs7,
}

#[derive(Parser, Debug)]
#[command(version, author, name = "check-sb-signature",
          about = "pkcs7 signature checker for efi binaries and efi variable updates",
          long_about = None)]
pub struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    pub loglevel: log::Level,

    /// pkcs7 verification code
    #[arg(long, hide = true)]
    pub code: Option<EfiCmsImpl>,

    /// enable asn1 dump
    #[arg(long, hide = true)]
    pub dump: bool,

    /// select 'PK' option
    #[arg(long, value_name = "OPT", help_heading = "certificate configuration")]
    pub pk_enroll: Option<PkEnroll>,

    /// use default 'KEK' certificates
    #[arg(long, help_heading = "certificate configuration")]
    pub default_kek: bool,

    /// select 'db' profile
    #[arg(long, value_name = "NAME", help_heading = "certificate configuration")]
    pub profile: Option<SecureBootProfile>,

    /// load additional certificates
    #[arg(long, value_name = "FILE", help_heading = "certificate configuration")]
    pub cert: Vec<String>,

    /// pe/efi binary to inspect
    #[arg(
        long,
        visible_alias = "efi",
        value_name = "FILE",
        help_heading = "verify signatures"
    )]
    pub pe: Vec<String>,

    /// KEK variable update to inspect
    #[arg(long, value_name = "FILE", help_heading = "verify signatures")]
    pub kek: Vec<String>,

    /// db variable update to inspect
    #[arg(long, value_name = "FILE", help_heading = "verify signatures")]
    pub db: Vec<String>,

    /// dbx variable update to inspect
    #[arg(long, value_name = "FILE", help_heading = "verify signatures")]
    pub dbx: Vec<String>,

    #[arg(long, hide = true)]
    pub manpage: bool,
}
