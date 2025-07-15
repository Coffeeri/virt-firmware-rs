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
    let pk = EfiVar {
        guid: guids::EfiGlobalVariable,
        name: SB_PK.to_string(),
        attr: EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        data: DATA_SB_PK.to_vec(),
    };
    let kek = EfiVar {
        guid: guids::EfiGlobalVariable,
        name: SB_KEK.to_string(),
        attr: EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        data: DATA_SB_KEK.to_vec(),
    };
    let db = EfiVar {
        guid: guids::EfiImageSecurityDatabase,
        name: SB_DB.to_string(),
        attr: EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        data: DATA_SB_DB.to_vec(),
    };
    let dbx = EfiVar {
        guid: guids::EfiImageSecurityDatabase,
        name: SB_DBX.to_string(),
        attr: EfiVarAttr::new_nv_bs_rt().with_time_auth_wr_access(true),
        data: DATA_SB_DBX.to_vec(),
    };

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
