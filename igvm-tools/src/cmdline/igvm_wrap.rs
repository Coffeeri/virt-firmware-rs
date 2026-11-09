use clap::{Parser, ValueHint};

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-wrap",
          about = "wrap classic native firmware image as igvm",
          long_about = None)]
pub struct Args {
    /// wrap TDVF for a TDVF-aware TDX loader
    #[arg(long, help_heading = "IGVM build options",
        conflicts_with_all = ["native", "snp", "vars", "real16", "flat32", "params", "no_params"])]
    pub tdx: bool,

    /// enable native platform (default)
    #[arg(
        long,
        help_heading = "IGVM build options",
        conflicts_with = "no_native"
    )]
    pub native: bool,

    /// disable native platform
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "native")]
    pub no_native: bool,

    /// enable sev-snp platform
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "no_snp")]
    pub snp: bool,

    /// disable sev-snp platform (default)
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "snp")]
    pub no_snp: bool,

    /// generate parameters (default)
    #[arg(
        long,
        help_heading = "IGVM build options",
        conflicts_with = "no_params"
    )]
    pub params: bool,

    /// do not generate parameters
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "params")]
    pub no_params: bool,

    /// boot firmware in real mode (default)
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "flat32")]
    pub real16: bool,

    /// boot firmware in protected mode
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "real16")]
    pub flat32: bool,

    /// print ovmf metadata
    #[arg(long, help_heading = "Print informations")]
    pub meta: bool,

    /// inspect generated igvm
    #[arg(long, help_heading = "Print informations")]
    pub inspect: bool,

    /// input firmware image
    #[arg(short, long, value_name = "CODE", value_hint = ValueHint::FilePath)]
    pub input: Option<String>,

    /// input uefi vars image
    #[arg(long, value_name = "VARS", value_hint = ValueHint::FilePath)]
    pub vars: Option<String>,

    /// output igvm image
    #[arg(short, long, value_name = "IGVM", value_hint = ValueHint::FilePath)]
    pub output: Option<String>,
}

impl Args {
    fn evaluate_bool(enable: bool, disable: bool, default: bool) -> bool {
        if enable {
            true
        } else if disable {
            false
        } else {
            default
        }
    }

    pub fn get_native(&self) -> bool {
        Args::evaluate_bool(self.native, self.no_native, !self.tdx)
    }

    pub fn get_snp(&self) -> bool {
        Args::evaluate_bool(self.snp, self.no_snp, false)
    }

    pub fn get_params(&self) -> bool {
        Args::evaluate_bool(self.params, self.no_params, true)
    }

    pub fn get_flat32(&self) -> bool {
        Args::evaluate_bool(self.flat32, self.real16, false)
    }
}
