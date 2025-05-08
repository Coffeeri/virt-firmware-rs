#![allow(dead_code)] // temporary

//!
//! [WIP] VarCheckPolicyLibMmiHandler + Edk2VariablePolicyProtocol reimplementation
//!
use alloc::string::String;
use alloc::vec::Vec;
use log::{debug, error};
use uguid::Guid;
use zerocopy::byteorder::little_endian::{U32, U64};
use zerocopy::{FromBytes, IntoBytes, Unalign};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, Unaligned};

use uefi::{Error, Status};
use virtfw_libefi::efivar::types::EfiVarAttr;

use crate::store::EfiVarStore;

// ----- VarCheckPolicyLibMmiHandler structs -----

const VAR_CHECK_POLICY_COMMAND_DISABLE: u32 = 0x01;
const VAR_CHECK_POLICY_COMMAND_IS_ENABLED: u32 = 0x02;
const VAR_CHECK_POLICY_COMMAND_REGISTER: u32 = 0x03;
const VAR_CHECK_POLICY_COMMAND_DUMP: u32 = 0x04;
const VAR_CHECK_POLICY_COMMAND_LOCK: u32 = 0x05;

// VAR_CHECK_POLICY_COMM_HEADER
#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, Unaligned, Clone)]
struct MmVarCheckPolicy {
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

// VAR_CHECK_POLICY_COMM_IS_ENABLED_PARAMS
#[repr(C, packed)]
#[derive(Debug, IntoBytes, Immutable)]
struct MmVarCheckPolicyEnabled {
    state: u8,
}

// ----- Edk2VariablePolicyProtocol structs -----

const VARIABLE_POLICY_ENTRY_REVISION: u32 = 0x00010000;

const VARIABLE_POLICY_TYPE_NO_LOCK: u32 = 0;
const VARIABLE_POLICY_TYPE_LOCK_NOW: u32 = 1;
const VARIABLE_POLICY_TYPE_LOCK_ON_CREATE: u32 = 2;
const VARIABLE_POLICY_TYPE_LOCK_ON_VAR_STATE: u32 = 3;

// VARIABLE_POLICY_ENTRY
#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct VariablePolicyEntry {
    version: u32,
    size: u16,
    offset_to_name: u16,
    namespace: [u8; 16],
    min_size: u32,
    max_size: u32,
    attr_must_have: u32,
    attr_cant_have: u32,
    lock_policy_type: u8,
    _padding: [u8; 3],
    // LockPolicy
    // Name
}

impl VariablePolicyEntry {
    pub fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read VariablePolicyEntry",
        )))
    }
}

// VARIABLE_LOCK_ON_VAR_STATE_POLICY
#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct VariableLockOnVarState {
    namespace: [u8; 16],
    value: u8,
    _padding: u8,
    // Name
}

impl VariableLockOnVarState {
    pub fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read VariableLockOnVarState",
        )))
    }
}

// ----- implementation structs -----

#[derive(Debug)]
pub struct ParsedVariablePolicyEntry {
    pub namespace: Guid,
    pub name: String,
    pub min_size: u32,
    pub max_size: u32,
    pub attr_must_have: EfiVarAttr,
    pub attr_cant_have: EfiVarAttr,
}

#[derive(Debug)]
pub struct MmVarPolicyRequest<'a> {
    hdr: MmVarCheckPolicy,
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
