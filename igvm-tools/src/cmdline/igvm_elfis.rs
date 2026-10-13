use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-elfis",
          about = "convert elf images into igvm",
          long_about = None)]
pub struct Args {
    /// boot firmware in real mode (default)
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "flat32")]
    pub real16: bool,

    /// boot firmware in protected mode
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "real16")]
    pub flat32: bool,

    /// list regions in elf image
    #[arg(long, help_heading = "Print informations")]
    pub regions: bool,

    /// inspect generated igvm
    #[arg(long, help_heading = "Print informations")]
    pub inspect: bool,

    /// input elf image
    #[arg(short, long, value_name = "ELF", required_unless_present = "manpage")]
    pub input: Option<String>,

    /// input efi binary (usually linux kernel)
    #[arg(short, long, value_name = "EFI")]
    pub kernel: Option<String>,

    /// output igvm image
    #[arg(short, long, value_name = "IGVM")]
    pub output: Option<String>,

    #[arg(long, hide = true)]
    pub manpage: bool,
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

    pub fn get_flat32(&self) -> bool {
        Args::evaluate_bool(self.flat32, self.real16, false)
    }
}
