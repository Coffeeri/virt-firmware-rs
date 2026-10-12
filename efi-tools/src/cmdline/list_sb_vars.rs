use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, name = "list-sb-vars", about = "list secure boot databases", long_about = None)]
pub struct Args {
    /// verbose mode (print more certificate details)
    #[arg(short, long)]
    pub verbose: bool,
}
