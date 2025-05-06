use core::fmt::Display;
use log::error;
use uguid::Guid;
use zerocopy::FromBytes;
use zerocopy_derive::FromBytes;

use uefi::{Error, Status};
use virtfw_libefi::guids;

use crate::mm::variable::variable_request;
use crate::store::EfiVarStore;

// EFI_MM_COMMUNICATE_HEADER
#[repr(C, packed)]
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

impl<'d> MmCore<'d> {
    pub fn new(data: &'d [u8]) -> Result<Self, Error> {
        let (header, body) = MmCoreHeader::read_from_prefix(data)
            .or::<Error>(Err(Status::BAD_BUFFER_SIZE.into()))?;
        let guid = Guid::from_bytes(header.guid);
        let size = header.size as usize;
        let data = &body[..size];
        Ok(MmCore { guid, size, data })
    }
}

impl Display for MmCore<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}: {} bytes", guids::pretty_name(&self.guid), self.size)
    }
}

pub fn core_request(store: &EfiVarStore, req: &mut [u8]) {
    let res = MmCore::new(req);
    if let Err(e) = res {
        error!("parse core: {}", e);
        return;
    };

    let creq = res.unwrap();
    match creq.guid {
        guids::EfiSmmVariableProtocol => {
            // We'll go operate on a owned copy of the data for
            // security reasons, to avoid the guest being able to
            // change the buffer wile we are processing it.
            #[allow(clippy::unnecessary_to_owned)]
            let rsp = variable_request(store, &creq.data.to_vec());
            assert!(rsp.len() <= creq.data.len());
            let start = core::mem::size_of::<MmCoreHeader>();
            let end = start + rsp.len();
            req[start..end].copy_from_slice(&rsp);
        }
        _ => {
            error!("core/unknown: {}", guids::pretty_name(&creq.guid));
            panic!()
        }
    }
}
