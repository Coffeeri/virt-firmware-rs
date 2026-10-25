use clap::{Parser, ValueHint};

#[derive(Parser, Debug)]
#[command(version, author, name = "mini-bootcfg",
          about = "show and manage uefi boot entries", long_about = None)]
pub struct Args {
    /// set loglevel
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    pub loglevel: log::Level,

    /// print boot configuration
    #[arg(short, long)]
    pub show: bool,

    /// more verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// add boot entry for UKI image FILE
    #[arg(long, value_name = "FILE", help_heading = "Update UKI boot entries",
          value_hint = ValueHint::FilePath)]
    pub add_uki: Option<String>,

    /// remove boot entry for UKI image FILE
    #[arg(long, value_name = "FILE", help_heading = "Update UKI boot entries",
          value_hint = ValueHint::FilePath)]
    pub remove_uki: Option<String>,

    /// remove stale UKI entries
    #[arg(long, help_heading = "Update UKI boot entries")]
    pub cleanup_uki: bool,

    /// override cmd line for boot entry
    #[arg(long, value_name = "CMDLINE", help_heading = "Update UKI boot entries")]
    pub cmdline: Option<String>,

    /// boot is successful.
    #[arg(
        long = "boot-ok",
        visible_alias = "boot-successful",
        help_heading = "Update UKI boot entries"
    )]
    pub boot_successful: bool,

    /// update BOOT.CSV
    #[arg(long, help_heading = "Update UKI boot entries")]
    pub update_csv: bool,

    /// add boot entry to netboot URI
    #[arg(long, value_name = "URI", help_heading = "Update other boot entries",
          value_hint = ValueHint::Url)]
    pub add_uri: Option<String>,

    /// remove boot entry NNNN
    #[arg(long, value_name = "NNNN", help_heading = "Update other boot entries")]
    pub remove_entry: Option<String>,

    /// boot added entry once (via BootNext)
    #[arg(
        long = "boot-next",
        visible_alias = "once",
        help_heading = "Options for entry updates"
    )]
    pub boot_next: bool,

    /// place added entry at POS to BootOrder
    #[arg(long, value_name = "POS", help_heading = "Options for entry updates")]
    pub boot_order: Option<String>,

    /// do not actually update the configuration
    #[arg(long, help_heading = "Options for entry updates")]
    pub dry_run: bool,

    /// set title for boot entry
    #[arg(long, value_name = "TITLE", help_heading = "Options for entry updates")]
    pub title: Option<String>,

    /// use shim binary FILE
    #[arg(long, value_name = "FILE", help_heading = "Options for entry updates",
          value_hint = ValueHint::FilePath)]
    pub shim: Option<String>,

    /// print boot loader name
    #[arg(long, help_heading = "Print system information")]
    pub print_loader: bool,

    /// print efi stub info
    #[arg(long, help_heading = "Print system information")]
    pub print_stub_info: bool,

    /// print efi stub image
    #[arg(long, help_heading = "Print system information")]
    pub print_stub_image: bool,
}
