use core::fmt::Display;
use uguid::Guid;
use zerocopy::error::SizeError;
use zerocopy::FromBytes;
use zerocopy_derive::FromBytes;

use virtfw_libefi::guids;

// EFI_MM_COMMUNICATE_HEADER
#[repr(C)]
#[derive(Debug, FromBytes)]
pub struct MmCoreHeader {
    pub guid: [u8; 16],
    pub size: u64,
}

#[derive(Debug)]
pub struct MmCore<'d> {
    pub guid: Guid,
    pub size: usize,
    pub data: &'d [u8],
}

impl<'data> MmCore<'data> {
    pub fn new(data: &'data [u8]) -> Result<Self, SizeError<&'data [u8], MmCoreHeader>> {
        let (header, body) = MmCoreHeader::read_from_prefix(data)?;
        let guid = Guid::from_bytes(header.guid);
        let size = header.size as usize;
        let data = &body[0..size];
        Ok(MmCore { guid, size, data })
    }
}

impl Display for MmCore<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}: {} bytes", guids::pretty_name(&self.guid), self.size)
    }
}
