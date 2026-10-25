use clap::{Parser, ValueHint};

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-inspect",
          about = "inspect igvm file content",
          long_about = None)]
pub struct Args {
    /// print ovmf metadata
    #[arg(long)]
    pub meta: bool,

    /// files to inspect
    #[arg(value_hint = ValueHint::FilePath)]
    pub files: Vec<String>,
}
