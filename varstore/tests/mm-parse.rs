#![cfg(feature = "std")]

use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr, EfiVarId};
use virtfw_libefi::guids;
use virtfw_varstore::mm::variable::{MmVariableParsedRequest, MmVariableRequest};

fn verify_req(req: &MmVariableParsedRequest) {
    let buffer = Vec::<u8>::from(req);
    let compare = MmVariableRequest::new(&buffer).unwrap();
    assert_eq!(compare.parsed.unwrap(), *req);
}

#[test]
fn req_get_variable() {
    let id = EfiVarId::new("SecureBoot".into(), guids::EfiGlobalVariable);
    let req = MmVariableParsedRequest::GetVariable(id);
    verify_req(&req);
}

#[test]
fn req_get_next_variable() {
    let id = EfiVarId::new("SecureBoot".into(), guids::EfiGlobalVariable);
    let req = MmVariableParsedRequest::GetNextVariableName(id);
    verify_req(&req);
}

#[test]
fn req_set_variable() {
    let attr = EfiVarAttr::new_bs_rt();
    let var = EfiVar::new_from_bool(&guids::EfiGlobalVariable, "SecureBoot", attr, true);
    let req = MmVariableParsedRequest::SetVariable(var);
    verify_req(&req);
}

#[test]
fn req_query_variable_info() {
    let req = MmVariableParsedRequest::QueryVariableInfo;
    verify_req(&req);
}

#[test]
fn req_ready_to_boot() {
    let req = MmVariableParsedRequest::ReadyToBoot;
    verify_req(&req);
}

#[test]
fn req_exit_boot_service() {
    let req = MmVariableParsedRequest::ExitBootService;
    verify_req(&req);
}

#[test]
fn req_get_payload_size() {
    let req = MmVariableParsedRequest::GetPayloadSize;
    verify_req(&req);
}
