use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_varstore::store::EfiVarStore;

const DATA_SB_PK: &[u8] = include_bytes!("data/enroll/PK.auth");
const DATA_SB_KEK: &[u8] = include_bytes!("data/enroll/KEK.auth");
const DATA_SB_DB: &[u8] = include_bytes!("data/enroll/db.auth");
const DATA_SB_DBX: &[u8] = include_bytes!("data/enroll/dbx.auth");

#[test]
fn sb_enroll() {
    let pk = EfiVar::new_from_slice(
        ids::PK.into(),
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        DATA_SB_PK,
    );
    let kek = EfiVar::new_from_slice(
        ids::KEK.into(),
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        DATA_SB_KEK,
    );
    let db = EfiVar::new_from_slice(
        ids::DB.into(),
        EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        DATA_SB_DB,
    );
    let dbx = EfiVar::new_from_slice(
        ids::DBX.into(),
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
