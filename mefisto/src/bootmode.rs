#[derive(Debug, PartialEq)]
#[repr(u64)]
pub enum BootMode {
    Unknown = 0,
    IgvmSNP = 5,
    Real16 = 16,
    Flat32 = 32,
}

impl From<u64> for BootMode {
    fn from(bm: u64) -> Self {
        match bm {
            5 => BootMode::IgvmSNP,
            16 => BootMode::Real16,
            32 => BootMode::Flat32,
            _ => BootMode::Unknown,
        }
    }
}
