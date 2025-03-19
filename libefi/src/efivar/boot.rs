//! efi boot configuration

use std::fmt;
use uuid::Uuid;

use crate::efivar::devpath::DevPath;

// efi variable names
pub const BOOT_CURRENT: &str = "BootCurrent";
pub const BOOT_NEXT: &str = "BootNext";
pub const BOOT_ORDER: &str = "BootOrder";
pub const SECURE_BOOT: &str = "SecureBoot";

// struct for boot index
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct BootIndex(pub u16);

impl fmt::Display for BootIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

impl From<&BootIndex> for Vec<u8> {
    fn from(value: &BootIndex) -> Vec<u8> {
        let bytes = value.0.to_le_bytes();
        bytes.to_vec()
    }
}

// struct for boot order (boot index list)
#[derive(Debug)]
pub struct BootOrder(pub Vec<BootIndex>);

impl BootOrder {
    pub fn new(elem: &BootIndex) -> BootOrder {
        let v = vec![*elem];
        BootOrder(v)
    }

    pub fn empty() -> BootOrder {
        let v = Vec::new();
        BootOrder(v)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn position(&self, elem: &BootIndex) -> Option<usize> {
        (0..self.0.len()).find(|&pos| elem == self.0.get(pos).unwrap())
    }

    pub fn insert(&self, index: usize, elem: &BootIndex) -> BootOrder {
        let mut v = self.0.clone();
        if self.position(elem).is_none() && index <= v.len() {
            v.insert(index, *elem);
        }
        BootOrder(v)
    }

    pub fn remove(&self, index: usize) -> BootOrder {
        let mut v = self.0.clone();
        if index < v.len() {
            v.remove(index);
        }
        BootOrder(v)
    }

    pub fn remove_elem(&self, elem: &BootIndex) -> BootOrder {
        let mut v = self.0.clone();
        if let Some(index) = self.position(elem) {
            v.remove(index);
        }
        BootOrder(v)
    }
}

impl fmt::Display for BootOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: Vec<String> = self.0.iter().map(|a| format!("{}", a)).collect();
        write!(f, "{}", s.join(","))
    }
}

impl From<&BootOrder> for Vec<u8> {
    fn from(value: &BootOrder) -> Vec<u8> {
        value.0.iter().flat_map(Vec::from).collect()
    }
}

// structs and consts for boot entries
pub const LOAD_OPTION_ACTIVE: u32 = 0x01;
pub const LOAD_OPTION_FORCE_RECONNECT: u32 = 0x02;
pub const LOAD_OPTION_HIDDEN: u32 = 0x08;

pub const LOAD_OPTION_CATEGORY: u32 = 0x1f00;
pub const LOAD_OPTION_CATEGORY_BOOT: u32 = 0x1f00;
pub const LOAD_OPTION_CATEGORY_APP: u32 = 0x0100;

#[derive(Debug)]
pub enum BootEntryOptData {
    None,
    String { string: String },
    Uuid { uuid: Uuid },
    Data { bytes: Vec<u8> },
}

impl fmt::Display for BootEntryOptData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BootEntryOptData::String { string } => {
                write!(f, "\"{}\"", string)
            }
            BootEntryOptData::Uuid { uuid } => {
                write!(f, "{}", uuid)
            }
            BootEntryOptData::Data { bytes } => {
                write!(f, "{:?}", bytes)
            }
            BootEntryOptData::None => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct BootEntry {
    pub attributes: u32,
    pub title: String,
    pub devpath: DevPath,
    pub optdata: BootEntryOptData,
}

impl BootEntry {
    pub fn new_boot(title: &str, devpath: DevPath, optdata: Option<String>) -> BootEntry {
        BootEntry {
            attributes: LOAD_OPTION_ACTIVE,
            title: title.to_string(),
            devpath,
            optdata: match optdata {
                Some(s) => BootEntryOptData::String { string: s },
                None => BootEntryOptData::None,
            },
        }
    }

    pub fn is_active(&self) -> bool {
        (self.attributes & LOAD_OPTION_ACTIVE) != 0
    }

    pub fn is_hidden(&self) -> bool {
        (self.attributes & LOAD_OPTION_HIDDEN) != 0
    }

    fn category(&self) -> u32 {
        self.attributes & LOAD_OPTION_CATEGORY
    }

    pub fn is_boot(&self) -> bool {
        self.category() == LOAD_OPTION_CATEGORY_BOOT
    }

    pub fn is_app(&self) -> bool {
        self.category() == LOAD_OPTION_CATEGORY_APP
    }
}
