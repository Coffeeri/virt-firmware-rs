use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version,
    author,
    name = "generate-boot-csv",
    about = "create BOOT.CSV file for fallback.efi",
    long_about = r#"

Generate BOOT.CSV file for fallback.efi with entries for all UKIs
found in the $ESP/EFI/Linux.  Additional EFI binaries can be passed
via --add command line switch.

This utility intentionally does not try to autodetect mount points or
read EFI variables.  So it can safely be used in environments like the
anaconda installer (kickstart scripts) or image builders.  This
implies the utility can not autodetect where your ESP is, so you have
to use the --esp switch if your ESP is not mounted at /efi.

"#
)]
pub struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    pub loglevel: log::Level,

    /// set efi system partition location
    #[arg(long, value_name = "ESP")]
    pub esp: Option<String>,

    /// add efi binary
    #[arg(long, value_name = "EFI")]
    pub add: Vec<String>,

    /// override csv output location
    #[arg(long, value_name = "CSV")]
    pub csv: Option<String>,

    /// override kernel cmd line (default: /etc/kernel/cmdline)
    #[arg(long, value_name = "APPEND")]
    pub cmdline: Option<String>,
}
