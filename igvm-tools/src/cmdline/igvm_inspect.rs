use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-inspect",
          about = "inspect igvm file content",
          long_about = None)]
pub struct Args {
    /// print ovmf metadata
    #[arg(long)]
    pub meta: bool,

    /// files to inspect
    #[arg()]
    pub files: Vec<String>,
}
