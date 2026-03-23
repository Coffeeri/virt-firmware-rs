use std::collections::HashMap;

use virtfw_libefi::efivar::ids;
use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr};
use virtfw_libefi::sb::certs::profiles::SecureBootProfile;
use virtfw_varstore::fs::FsIndexParser;
use virtfw_varstore::store::EfiVarStore;

struct RamFS {
    storage: HashMap<u32, Vec<u8>>,
}

impl RamFS {
    fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    fn read(&self, inode: u32) -> Option<&[u8]> {
        let data = self.storage.get(&inode)?;
        Some(data)
    }

    fn write(&mut self, inode: u32, data: &[u8]) {
        self.storage.insert(inode, data.into());
    }

    fn remove(&mut self, inode: u32) {
        self.storage.remove(&inode);
    }

    fn list(&self) -> Vec<u32> {
        self.storage.keys().map(|inode| *inode).collect()
    }
}

fn make_test_store() -> EfiVarStore {
    let mut store = EfiVarStore::new();

    // secure boot setup (so we have some vars to test with)
    store.enroll_pk_mgmt();
    store.enroll_kek_microsoft();
    store.enroll_db_profile(SecureBootProfile::Uefi11);
    store.enroll_dbx_native();

    store
}

#[test]
fn fs_index() {
    let mut store = make_test_store();

    // try serialize and parse the variable index
    let index_blob_1 = store.fs_make_index().unwrap();
    let _: Vec<_> = FsIndexParser::new(&index_blob_1).collect();

    // update one variable
    store.enroll_db_profile(SecureBootProfile::RedHat);

    // try serialize + parse again
    let index_blob_2 = store.fs_make_index().unwrap();
    let index_parsed: Vec<_> = FsIndexParser::new(&index_blob_2).collect();

    // verify the vars are present
    index_parsed.iter().find(|(_e, n)| n == "PK").unwrap();
    index_parsed.iter().find(|(_e, n)| n == "KEK").unwrap();
    index_parsed.iter().find(|(_e, n)| n == "db").unwrap();
    index_parsed.iter().find(|(_e, n)| n == "dbx").unwrap();
}

// read store from filesystem
fn fs_read(store: &mut EfiVarStore, filesystem: &RamFS) -> u32 {
    let mut count = 0;

    let index = filesystem.read(store.fs_inode_index()).unwrap();
    for (entry, name) in FsIndexParser::new(&index) {
        let data = filesystem.read(entry.inode()).unwrap();
        store.fs_set_variable(&entry, &name, data);
        count += 1
    }

    count
}

// write store to filesystem
fn fs_write(store: &mut EfiVarStore, filesystem: &mut RamFS) -> u32 {
    let mut count = 0;

    let mods = store.fs_get_modified();
    if mods.is_unchanged() {
        return count;
    }

    let index = store.fs_make_index().unwrap();

    if mods.variable_added || mods.variable_updated {
        // write variables
        for (entry, name) in FsIndexParser::new(&index) {
            if let Some(data) = store.fs_get_variable(&entry, &name) {
                filesystem.write(entry.inode(), data);
                count += 1
            }
        }
    }

    if mods.variable_added || mods.variable_removed {
        // write index
        filesystem.write(store.fs_inode_index(), &index);
        count += 1;
    }

    store.fs_clear_modified();

    count
}

// garbage collect unused filesystem inodes
fn fs_gc(store: &mut EfiVarStore, filesystem: &mut RamFS) -> u32 {
    let mut count = 0;

    for inode in filesystem.list() {
        if store.fs_inode_is_used(inode) {
            continue;
        }
        filesystem.remove(inode);
        count += 1;
    }

    count
}

#[test]
fn fs_write_read() {
    let mut store1 = make_test_store();
    let mut filesystem = RamFS::new();

    // initial write
    let updates = fs_write(&mut store1, &mut filesystem);
    assert_eq!(updates, 5);

    let updates = fs_write(&mut store1, &mut filesystem);
    assert_eq!(updates, 0);

    // add SecureBootEnable + CustomMode
    store1.reset();
    let updates = fs_write(&mut store1, &mut filesystem);
    assert_eq!(updates, 3);

    // add SHIM_VERBOSE
    let sv = EfiVar::new_from_bool(ids::SHIM_VERBOSE.into(), EfiVarAttr::new_nv_bs_rt(), true);
    store1.set(sv.clone()).unwrap();
    let updates = fs_write(&mut store1, &mut filesystem);
    assert_eq!(updates, 2);

    // remove SHIM_VERBOSE
    store1.delete(sv).unwrap();
    let updates = fs_write(&mut store1, &mut filesystem);
    assert_eq!(updates, 1);
    let updates = fs_gc(&mut store1, &mut filesystem);
    assert_eq!(updates, 1);

    // read test
    let mut store2 = EfiVarStore::new();
    let entries = fs_read(&mut store2, &filesystem);
    assert_eq!(entries, 6);

    let updates = fs_write(&mut store2, &mut filesystem);
    assert_eq!(updates, 0);
}
