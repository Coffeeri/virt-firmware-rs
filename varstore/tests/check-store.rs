#![cfg(feature = "std")]

use uefi::Status;
use uguid::{guid, Guid};

use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_varstore::store::EfiVarStore;

#[allow(non_upper_case_globals)]
pub const EfiVarStoreTests: Guid = guid!("cb70b45c-ac88-430a-bc14-5f6b66b30277");

#[test]
fn read_write_nv_bs() {
    let tvar = EfiVar {
        guid: EfiVarStoreTests,
        name: "test-nv-bs".into(),
        attr: EfiVarAttr::new_nv_bs(),
        data: [23].to_vec(),
    };

    let mut store = EfiVarStore::new();
    store.set(tvar.clone()).expect("set failed");

    // boot service -> ok
    let var = store.get(&tvar.name, &tvar.guid).expect("get #1 failed");
    assert_eq!(*var, tvar);

    // runtime service -> not found (no runtime access)
    store.exit_boot_service();
    let res = store.get(&tvar.name, &tvar.guid);
    assert_eq!(res.err().unwrap().status(), Status::NOT_FOUND);

    // reset -> boot service -> ok
    store.reset();
    let var = store.get(&tvar.name, &tvar.guid).expect("get #2 failed");
    assert_eq!(*var, tvar);
}

#[test]
fn read_write_bs_rt() {
    let tvar = EfiVar {
        guid: EfiVarStoreTests,
        name: "test-bs-rt".into(),
        attr: EfiVarAttr::new_bs_rt(),
        data: [42].to_vec(),
    };

    let mut store = EfiVarStore::new();
    store.set(tvar.clone()).expect("set failed");

    // boot service -> ok
    let var = store.get(&tvar.name, &tvar.guid).expect("get #1 failed");
    assert_eq!(*var, tvar);

    // runtime service -> ok
    store.exit_boot_service();
    let var = store.get(&tvar.name, &tvar.guid).expect("get #2 failed");
    assert_eq!(*var, tvar);

    // reset -> not found (volatile vars removed)
    store.reset();
    let res = store.get(&tvar.name, &tvar.guid);
    assert_eq!(res.err().unwrap().status(), Status::NOT_FOUND);
}

#[test]
fn attr_reserved() {
    let tvar = EfiVar {
        guid: EfiVarStoreTests,
        name: "test-attr-reserved".into(),
        attr: EfiVarAttr::new_nv_bs().with_reserved(4711),
        data: [41, 11].to_vec(),
    };

    let mut store = EfiVarStore::new();
    let res = store.set(tvar.clone());
    assert_eq!(res.err().unwrap().status(), Status::INVALID_PARAMETER);
}

#[test]
fn attr_hw_error() {
    let tvar = EfiVar {
        guid: EfiVarStoreTests,
        name: "test-attr-hw-error".into(),
        attr: EfiVarAttr::new_nv_bs().with_hardware_error(true),
        data: [41, 12].to_vec(),
    };

    let mut store = EfiVarStore::new();
    let res = store.set(tvar.clone());
    assert_eq!(res.err().unwrap().status(), Status::INVALID_PARAMETER);
}

#[test]
fn attr_auth_obsolete() {
    let tvar = EfiVar {
        guid: EfiVarStoreTests,
        name: "test-attr-auth-obsolete".into(),
        attr: EfiVarAttr::new_nv_bs().with_auth_wr_access(true),
        data: [41, 13].to_vec(),
    };

    let mut store = EfiVarStore::new();
    let res = store.set(tvar.clone());
    assert_eq!(res.err().unwrap().status(), Status::UNSUPPORTED);
}

#[test]
fn lock_now() {
    let tvar = EfiVar {
        guid: EfiVarStoreTests,
        name: "test-lock-now".into(),
        attr: EfiVarAttr::new_nv_bs(),
        data: [11].to_vec(),
    };

    let mut store = EfiVarStore::new();
    store.set(tvar.clone()).expect("set failed");

    store.lock(&tvar.name, &tvar.guid).expect("lock failed");
    store.end_of_dxe();

    let res = store.set(tvar.clone());
    assert_eq!(res.err().unwrap().status(), Status::WRITE_PROTECTED);
}
