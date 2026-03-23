//!
//! read/write varstore to filesystem (coconfs) storage
//!
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use uguid::Guid;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use virtfw_libefi::efivar::types::{EfiVar, EfiVarAttr, EfiVarId};
use virtfw_libefi::guids::RawGuid;

use crate::store::{EfiVarStore, EfiVarStoreChanges, VariableEntry};

const INODE_NR_INDEX: u32 = 0;
const INODE_NR_START: u32 = 16;

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Clone)]
pub struct FsVariableIndexEntry {
    guid: RawGuid,
    inode: u32,
    attributes: EfiVarAttr,
    name_bytes: u32,
}

impl FsVariableIndexEntry {
    pub fn inode(&self) -> u32 {
        self.inode
    }
}

impl TryFrom<&VariableEntry> for FsVariableIndexEntry {
    type Error = Box<dyn core::error::Error>;

    fn try_from(var: &VariableEntry) -> Result<Self, Box<dyn core::error::Error>> {
        if let Some(inode) = var.inode {
            let name_bytes = var.var.name().len() as u32;
            Ok(Self {
                guid: var.var.guid().into(),
                attributes: var.var.attr(),
                name_bytes,
                inode,
            })
        } else {
            Err("no inode number assigned".into())
        }
    }
}

pub struct FsIndexParser<'i> {
    offset: usize,
    index: &'i [u8],
}

impl<'i> FsIndexParser<'i> {
    pub fn new(index: &'i [u8]) -> Self {
        Self { offset: 0, index }
    }
}

impl<'i> Iterator for FsIndexParser<'i> {
    type Item = (FsVariableIndexEntry, String);

    fn next(&mut self) -> Option<Self::Item> {
        let (entry, r) = FsVariableIndexEntry::read_from_prefix(&self.index[self.offset..]).ok()?;
        let name_bytes = entry.name_bytes as usize;
        let name_blob = r.get(0..name_bytes)?;
        let name = str::from_utf8(name_blob).ok()?;
        self.offset += size_of::<FsVariableIndexEntry>();
        self.offset += name_bytes;
        Some((entry, name.into()))
    }
}

impl EfiVarStore {
    fn fs_assign_inodes(&mut self) {
        let mut imax = self
            .variables
            .iter()
            .filter_map(|v| v.inode)
            .max()
            .unwrap_or(INODE_NR_START - 1);
        for var in self
            .variables
            .iter_mut()
            .filter(|v| v.attr().non_volatile())
            .filter(|v| v.inode.is_none())
        {
            imax += 1;
            var.inode = Some(imax);
        }
    }

    pub fn fs_make_index(&mut self) -> Result<Vec<u8>, Box<dyn core::error::Error>> {
        self.fs_assign_inodes();
        let mut blob = Vec::new();
        for var in self.variables.iter().filter(|v| v.attr().non_volatile()) {
            let entry: FsVariableIndexEntry = var.try_into()?;
            blob.extend_from_slice(entry.as_bytes());
            blob.extend_from_slice(var.name().as_bytes());
        }
        Ok(blob)
    }

    pub fn fs_get_variable(&self, entry: &FsVariableIndexEntry, name: &str) -> Option<&[u8]> {
        let guid = Guid::from(entry.guid);
        let var = self
            .variables
            .iter()
            .filter(|v| v.attr().non_volatile())
            .find(|v| v.guid() == &guid && v.name() == name)?;
        if !var.modified {
            return None;
        }
        Some(var.data())
    }

    pub fn fs_set_variable(&mut self, entry: &FsVariableIndexEntry, name: &str, data: &[u8]) {
        let id = EfiVarId::new(name.into(), entry.guid.into());
        let index = self.get_position(&id);
        if let Some(i) = index {
            self.variables.remove(i);
        }

        let var = EfiVar::new_from_slice(id, entry.attributes, data);
        let entry = VariableEntry::new(var, Some(entry.inode), false);
        self.variables.push(entry);
    }

    pub fn fs_inode_index(&self) -> u32 {
        INODE_NR_INDEX
    }

    pub fn fs_inode_is_used(&self, inode: u32) -> bool {
        if inode == INODE_NR_INDEX {
            return true;
        }
        self.variables
            .iter()
            .filter(|v| v.attr().non_volatile())
            .find(|v| v.inode == Some(inode))
            .is_some()
    }

    pub fn fs_get_modified(&self) -> EfiVarStoreChanges {
        self.modified.clone()
    }

    pub fn fs_clear_modified(&mut self) {
        for v in self.variables.iter_mut() {
            v.modified = false;
        }
        self.modified.clear();
    }
}
