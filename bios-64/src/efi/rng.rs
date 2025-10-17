use log::debug;

pub struct Random {}

impl Random {
    pub fn new() -> Option<Self> {
        Some(Self {})
    }
}

impl efiloader::Random for Random {
    fn get_entropy(&self, _bytes: &mut [u8], use_raw: bool) -> bool {
        debug!("get_entropy: raw={use_raw}");
        false
    }
}
