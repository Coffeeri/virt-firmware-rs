use alloc::vec::Vec;
use log::{debug, error};
use zerocopy::byteorder::little_endian::{U32, U64};
use zerocopy::{FromBytes, IntoBytes, Unalign};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, Unaligned};

use uefi::{Error, Status};

use crate::store::EfiVarStore;

pub const VAR_CHECK_POLICY_COMMAND_DISABLE: u32 = 0x01;
pub const VAR_CHECK_POLICY_COMMAND_IS_ENABLED: u32 = 0x02;
pub const VAR_CHECK_POLICY_COMMAND_REGISTER: u32 = 0x03;
pub const VAR_CHECK_POLICY_COMMAND_DUMP: u32 = 0x04;
pub const VAR_CHECK_POLICY_COMMAND_LOCK: u32 = 0x05;

// VAR_CHECK_POLICY_COMM_HEADER
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, Unaligned, Clone)]
pub struct MmVarCheckPolicy {
    signature: u32,
    revision: u32,
    command: Unalign<U32>,
    result: Unalign<U64>,
}

impl MmVarCheckPolicy {
    pub fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read MmVarCheckPolicy",
        )))
    }
}

// request state
#[derive(Debug)]
pub struct MmVarPolicyRequest<'a> {
    pub hdr: MmVarCheckPolicy,
    pub data: &'a [u8],
}

impl<'a> MmVarPolicyRequest<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error<&'static str>> {
        let (hdr, _body) = MmVarCheckPolicy::new(data)?;
        Ok(MmVarPolicyRequest { hdr, data })
    }

    fn response(&self, status: Status, _reply: Option<&[u8]>) -> Vec<u8> {
        debug!("policy/rsp: {}", status);

        let mut hdr = self.hdr.clone();
        hdr.result.set((status.0 as u64).into());

        let rsp = hdr.as_bytes().to_vec();
        rsp
    }

    fn process(&self, _store: &mut EfiVarStore) -> Vec<u8> {
        let cmd = self.hdr.command.get();
        if cmd == VAR_CHECK_POLICY_COMMAND_REGISTER {
            debug!("policy/req: register [TODO]");
            self.response(Status::SUCCESS, None)
        } else {
            debug!("policy/req: command 0x{:x} [TODO]", cmd);
            Vec::new()
        }
    }
}

pub fn policy_request(store: &mut EfiVarStore, req: &[u8]) -> Vec<u8> {
    let res = MmVarPolicyRequest::new(req);
    if let Err(e) = res.as_ref() {
        error!("{e}");
        return Vec::new();
    }

    let vreq = res.unwrap();
    vreq.process(store)
}
