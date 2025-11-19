//!
//! EFI_MM_COMMUNICATE parser
//!
use alloc::vec::Vec;
use core::fmt::Display;
use log::error;
use uguid::Guid;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use uefi::{Error, Status};
use virtfw_libefi::guids;

use crate::mm::policy::policy_request;
use crate::mm::variable::variable_request;
use crate::store::EfiVarStore;

// EFI_MM_COMMUNICATE_HEADER
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct MmCoreHeader {
    pub guid: [u8; 16],
    pub size: u64,
}

impl MmCoreHeader {
    pub fn new(guid: Guid, size: u64) -> Self {
        Self {
            guid: guid.to_bytes(),
            size,
        }
    }
}

#[derive(Debug)]
pub struct MmCore<'d> {
    pub guid: [u8; 16],
    pub data: &'d [u8],
}

impl<'d> MmCore<'d> {
    pub fn new(data: &'d [u8]) -> Result<Self, Error> {
        let (header, body) = MmCoreHeader::read_from_prefix(data)
            .or::<Error>(Err(Status::BAD_BUFFER_SIZE.into()))?;
        let size = header.size as usize;
        let Some(data) = &body.get(..size) else {
            return Err(Status::BAD_BUFFER_SIZE.into());
        };
        Ok(MmCore {
            guid: header.guid,
            data,
        })
    }
}

impl Display for MmCore<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let guid = Guid::from_bytes(self.guid);
        write!(
            f,
            "{}: {} bytes",
            guids::pretty_name(&guid),
            self.data.len()
        )
    }
}

pub fn core_request_dispatch(
    store: &mut EfiVarStore,
    guid_bytes: &[u8; 16],
    req: &[u8],
) -> Vec<u8> {
    let guid = Guid::from_bytes(*guid_bytes);
    match guid {
        guids::EfiSmmVariableProtocol => variable_request(store, req),
        guids::VarCheckPolicyLibMmiHandler => policy_request(store, req),
        guids::EfiEndOfDxeEventGroup => {
            store.end_of_dxe();
            Vec::new()
        }
        guids::EfiEventReadyToBoot => {
            store.ready_to_boot();
            Vec::new()
        }
        guids::EfiEventExitBootServices => {
            store.exit_boot_service();
            Vec::new()
        }
        _ => {
            error!("core/unknown: {}", guids::pretty_name(&guid));
            Vec::new()
        }
    }
}

pub fn core_request(store: &mut EfiVarStore, req: &mut [u8]) {
    let res = MmCore::new(req);
    if let Err(e) = res {
        error!("parse core: {e}");
        return;
    };

    let creq = res.unwrap();

    // We'll go operate on a owned copy of the data for
    // security reasons, to avoid the guest being able to
    // change the buffer wile we are processing it.
    #[allow(clippy::unnecessary_to_owned)]
    let rsp = core_request_dispatch(store, &creq.guid, &creq.data.to_vec());

    // copy result back to request buffer
    assert!(rsp.len() <= creq.data.len());
    let start = core::mem::size_of::<MmCoreHeader>();
    let end = start + rsp.len();
    req[start..end].copy_from_slice(&rsp);
}
