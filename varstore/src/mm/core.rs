use uguid::Guid;
use zerocopy::FromBytes;
use zerocopy_derive::FromBytes;

// EFI_MM_COMMUNICATE_HEADER
#[repr(C)]
#[derive(Debug,FromBytes)]
pub struct MmCoreHeader {
    pub guid: Guid,
    pub size: u64,
}

impl Mm
