use alloc::vec::Vec;
use log::{debug, error};

use uefi::Error;

use crate::store::EfiVarStore;

// request state
#[derive(Debug)]
pub struct MmVarPolicyRequest<'a> {
    pub data: &'a [u8],
}

impl<'a> MmVarPolicyRequest<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error<&'static str>> {
        Ok(MmVarPolicyRequest { data })
    }

    fn process(&self, _store: &mut EfiVarStore) -> Vec<u8> {
        debug!("TODO: VarCheckPolicyLibMmiHandler");
        Vec::new()
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
