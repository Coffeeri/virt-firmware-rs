use virtfw_libhw::x86cpuid::{has_rdrand, rdrand};

pub struct Random {
    enabled: bool,
}

impl Random {
    pub fn new() -> Option<Self> {
        let enabled = has_rdrand();
        Some(Self { enabled })
    }
}

impl efiloader::Random for Random {
    fn get_entropy(&self, bytes: &mut [u8], _use_raw: bool) -> bool {
        if !self.enabled {
            return false;
        }

        let mut b: &mut [u8] = bytes;
        while !b.is_empty() {
            let r = rdrand();
            let n = b.len().min(core::mem::size_of_val(&r));
            let v: &mut [u8];
            (v, b) = b.split_at_mut(n);
            v.copy_from_slice(&r.to_le_bytes()[..n]);
        }

        true
    }
}
