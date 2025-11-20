use crate::print;

pub struct Console {}

impl Console {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl efiloader::SimpleConsole for Console {
    fn write_string(&self, s: &str) {
        print!("{s}");
    }

    fn read_byte(&self) -> Option<u8> {
        None
    }
}
