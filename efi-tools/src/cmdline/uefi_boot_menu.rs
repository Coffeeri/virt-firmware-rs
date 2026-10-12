use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, name = "uefi-boot-menu", about = None, long_about = None)]
pub struct Args {
    /// enable debug logging
    #[arg(short, long)]
    pub debug: bool,

    /// reboot after picking an entry
    #[arg(short, long)]
    pub reboot: bool,
}
