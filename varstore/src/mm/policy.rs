//!
//! [WIP] VarCheckPolicyLibMmiHandler + Edk2VariablePolicyProtocol reimplementation
//!
use alloc::string::String;
use alloc::vec::Vec;
use log::{debug, error};
use uguid::Guid;
use zerocopy::byteorder::little_endian::{U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, Unalign, Unaligned};

use uefi::{Error, Status};

use crate::mm::util;
use crate::policy::{VariablePolicyEntry, VariablePolicyType};
use crate::store::EfiVarStore;

// ----- VarCheckPolicyLibMmiHandler structs -----

const VAR_CHECK_POLICY_COMMAND_DISABLE: u32 = 0x01;
const VAR_CHECK_POLICY_COMMAND_IS_ENABLED: u32 = 0x02;
const VAR_CHECK_POLICY_COMMAND_REGISTER: u32 = 0x03;
//const VAR_CHECK_POLICY_COMMAND_DUMP: u32 = 0x04;
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

const VARIABLE_POLICY_TYPE_NO_LOCK: u8 = 0;
const VARIABLE_POLICY_TYPE_LOCK_NOW: u8 = 1;
const VARIABLE_POLICY_TYPE_LOCK_ON_CREATE: u8 = 2;
const VARIABLE_POLICY_TYPE_LOCK_ON_VAR_STATE: u8 = 3;

// VARIABLE_POLICY_ENTRY
#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct MmVariablePolicyEntry {
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

impl MmVariablePolicyEntry {
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
struct MmVariableLockOnVarState {
    namespace: [u8; 16],
    value: u8,
    _padding: u8,
    // Name
}

impl MmVariableLockOnVarState {
    pub fn new(data: &[u8]) -> Result<(Self, &[u8]), Error<&'static str>> {
        Self::read_from_prefix(data).or(Err(Error::new(
            Status::BAD_BUFFER_SIZE,
            "read VariableLockOnVarState",
        )))
    }
}

// ----- implementation structs -----

#[derive(Debug)]
pub struct MmVarPolicyRequest<'a> {
    hdr: MmVarCheckPolicy,
    pub body: &'a [u8],
}

impl<'a> MmVarPolicyRequest<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error<&'static str>> {
        let (hdr, body) = MmVarCheckPolicy::new(data)?;
        Ok(MmVarPolicyRequest { hdr, body })
    }

    fn response(&self, status: Status, _reply: Option<&[u8]>) -> Vec<u8> {
        debug!("policy/rsp: {}", status);

        let mut hdr = self.hdr.clone();
        hdr.result.set((status.0 as u64).into());

        let rsp = hdr.as_bytes().to_vec();
        rsp
    }

    fn parse_entry(&self) -> Result<VariablePolicyEntry, Error<&'static str>> {
        let (policy, _) = MmVariablePolicyEntry::new(self.body)?;
        if policy.version != VARIABLE_POLICY_ENTRY_REVISION {
            return Err(Error::new(Status::UNSUPPORTED, "unknown policy version"));
        }
        let pst = core::mem::size_of::<MmVariablePolicyEntry>();
        let nst = policy.offset_to_name as usize;
        let end = policy.size as usize;
        let name = util::cstr16_from_bytes_with_nul(&self.body[nst..end])?;
        let policy_type = match policy.lock_policy_type {
            VARIABLE_POLICY_TYPE_NO_LOCK => VariablePolicyType::NoLock,
            VARIABLE_POLICY_TYPE_LOCK_NOW => VariablePolicyType::LockNow,
            VARIABLE_POLICY_TYPE_LOCK_ON_CREATE => VariablePolicyType::LockOnCreate,
            VARIABLE_POLICY_TYPE_LOCK_ON_VAR_STATE => {
                let (vstate, more) = MmVariableLockOnVarState::new(&self.body[pst..nst])?;
                let vname = util::cstr16_from_bytes_with_nul(more)?;
                VariablePolicyType::LockOnVarState {
                    namespace: Guid::from_bytes(vstate.namespace),
                    name: String::from(vname),
                    value: vstate.value,
                }
            }
            _ => {
                return Err(Error::new(
                    Status::INVALID_PARAMETER,
                    "invalid variable policy type",
                ))
            }
        };

        Ok(VariablePolicyEntry {
            namespace: Guid::from_bytes(policy.namespace),
            name: String::from(name),
            min_size: policy.min_size,
            max_size: policy.max_size,
            attr_must_have: policy.attr_must_have.into(),
            attr_cant_have: policy.attr_cant_have.into(),
            lock_policy_type: policy_type,
        })
    }

    fn process(&self, store: &mut EfiVarStore) -> Vec<u8> {
        let cmd: u32 = self.hdr.command.get().into();
        match cmd {
            VAR_CHECK_POLICY_COMMAND_DISABLE => {
                debug!("policy/req/disable");
                store.policy_disable();
                self.response(Status::SUCCESS, None)
            }
            VAR_CHECK_POLICY_COMMAND_IS_ENABLED => {
                let enabled = MmVarCheckPolicyEnabled {
                    state: if store.policy_is_enabled() { 1 } else { 0 },
                };
                self.response(Status::SUCCESS, Some(enabled.as_bytes()))
            }
            VAR_CHECK_POLICY_COMMAND_REGISTER => {
                let res = self.parse_entry();
                if let Err(e) = res {
                    return self.response(e.status(), None);
                }
                let policy = res.unwrap();
                debug!("policy/req/register: {}", policy);
                let res = store.policy_register(policy);
                if let Err(e) = res {
                    return self.response(e.status(), None);
                }
                self.response(Status::SUCCESS, None)
            }
            VAR_CHECK_POLICY_COMMAND_LOCK => {
                debug!("policy/req/lock");
                store.policy_lock();
                self.response(Status::SUCCESS, None)
            }
            _ => {
                debug!("policy/req: unsupported command 0x{:x}", cmd);
                self.response(Status::UNSUPPORTED, None)
            }
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
