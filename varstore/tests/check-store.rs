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
        attr: EfiVarAttr::new()
            .with_non_volatile(true)
            .with_bootservice_access(true),
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
        attr: EfiVarAttr::new()
            .with_bootservice_access(true)
            .with_runtime_access(true),
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
