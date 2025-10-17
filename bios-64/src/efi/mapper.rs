use core::ops::Range;
use log::debug;

pub struct MemoryMapper {}

impl MemoryMapper {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MemoryMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl efiloader::MemoryMapper for MemoryMapper {
    fn remap_range(&self, range: &Range<usize>, set: u64, clr: u64) -> Result<(), &str> {
        debug!(
            "remap: range=0x{:x}-0x{:x} set=0x{set:x} clr=0x{clr:x}",
            range.start, range.end
        );
        Ok(())
    }

    fn query_range(&self, range: &Range<usize>) -> Option<u64> {
        debug!("query: range=0x{:x}-0x{:x}", range.start, range.end);
        None
    }
}
