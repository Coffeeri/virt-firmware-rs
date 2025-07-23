use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::guids;
use virtfw_varstore::authvar::{SB_DB, SB_DBX, SB_KEK, SB_PK};
use virtfw_varstore::store::EfiVarStore;

const DATA_SB_PK: &[u8] = include_bytes!("data/enroll/PK.auth");
const DATA_SB_KEK: &[u8] = include_bytes!("data/enroll/KEK.auth");
const DATA_SB_DB: &[u8] = include_bytes!("data/enroll/db.auth");
const DATA_SB_DBX: &[u8] = include_bytes!("data/enroll/dbx.auth");

#[test]
fn sb_enroll() {
    let pk = EfiVar::new_from_slice(
        &guids::EfiGlobalVariable,
        SB_PK,
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        DATA_SB_PK,
    );
    let kek = EfiVar::new_from_slice(
        &guids::EfiGlobalVariable,
        SB_KEK,
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        DATA_SB_KEK,
    );
    let db = EfiVar::new_from_slice(
        &guids::EfiImageSecurityDatabase,
        SB_DB,
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        DATA_SB_DB,
    );
    let dbx = EfiVar::new_from_slice(
        &guids::EfiImageSecurityDatabase,
        SB_DBX,
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        DATA_SB_DBX,
    );

    let mut store = EfiVarStore::new();

    // Enroll secure boot certificate databases.
    // Setting PK turns off setup mode, so that must come last.
    let res = store.set(db.clone());
    assert!(res.is_ok());
    let res = store.set(dbx.clone());
    assert!(res.is_ok());
    let res = store.set(kek.clone());
    assert!(res.is_ok());
    let res = store.set(pk.clone());
    assert!(res.is_ok());

    // Try override (must fail).
    let res = store.set(db.clone());
    assert!(res.is_err());
    let res = store.set(dbx.clone());
    assert!(res.is_err());
    let res = store.set(kek.clone());
    assert!(res.is_err());
    let res = store.set(pk.clone());
    assert!(res.is_err());
}
