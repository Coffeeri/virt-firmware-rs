extern crate std;
use std::collections::HashMap;
use std::collections::HashSet;
use std::format;
use std::io::Cursor;
use std::string::String;
use std::vec::Vec;

use log::error;

use crate::efivar::boot::{BootEntry, BootIndex, BootOrder};
use crate::efivar::io::ReadEfiExt;
use crate::guids;
use crate::varstore::sysfs;

pub struct BootConfig {
    pub sb: bool,
    pub current: Option<BootIndex>,
    pub next: Option<BootIndex>,
    pub order: Option<BootOrder>,
    pub entries: HashMap<BootIndex, BootEntry>,
}

impl BootConfig {
    fn read_bool(name: &str) -> bool {
        if let Some(var) = sysfs::varstore_read(name, &guids::EfiGlobalVariable) {
            if var.data.len() == 1 {
                return var.data[0] != 0;
            }
        }
        false
    }

    fn read_index(name: &str) -> Option<BootIndex> {
        let var = sysfs::varstore_read(name, &guids::EfiGlobalVariable)?;
        let mut r = Cursor::new(var.data);
        let index = r.read_boot_index();
        if let Err(e) = index {
            error!("failed to parse {}: {}", name, e);
            return None;
        };
        Some(index.unwrap())
    }

    fn read_order() -> Option<BootOrder> {
        let var = sysfs::varstore_read("BootOrder", &guids::EfiGlobalVariable)?;
        let mut r = Cursor::new(var.data);
        let order = r.read_boot_order();
        if let Err(e) = order {
            error!("failed to parse BootOrder: {}", e);
            return None;
        };
        Some(order.unwrap())
    }

    fn read_entries() -> HashMap<BootIndex, BootEntry> {
        let mut entries = HashMap::new();
        let vars = sysfs::varstore_ls().expect("failed to list efi vars");

        for var in vars {
            if var.guid != guids::EfiGlobalVariable {
                continue;
            }
            if !var.name.starts_with("Boot") {
                continue;
            }

            let index_str = var.name.get(4..).unwrap();
            let Ok(index_u16) = u16::from_str_radix(index_str, 16) else {
                continue;
            };
            let index = BootIndex(index_u16);

            let Some(var) = sysfs::varstore_read_id(&var) else {
                continue;
            };
            let mut r = Cursor::new(var.data);
            let entry = r.read_boot_entry();
            if let Err(e) = entry {
                error!("failed to parse {}: {}", var.name, e);
                continue;
            };

            entries.insert(index, entry.unwrap());
        }
        entries
    }

    pub fn new() -> BootConfig {
        let sb = BootConfig::read_bool("SecureBoot");
        let current = BootConfig::read_index("BootCurrent");
        let next = BootConfig::read_index("BootNext");
        let order = BootConfig::read_order();
        let entries = BootConfig::read_entries();

        BootConfig {
            sb,
            current,
            next,
            order,
            entries,
        }
    }

    pub fn index_list(&self) -> Vec<&BootIndex> {
        let mut list = Vec::new();
        let mut seen = HashSet::new();

        if let Some(n) = &self.next {
            list.push(n);
            seen.insert(n);
        }

        if let Some(c) = &self.current {
            if !seen.contains(c) {
                list.push(c);
                seen.insert(c);
            }
        }

        if let Some(o) = &self.order {
            for i in &o.0 {
                if !seen.contains(i) {
                    list.push(i);
                    seen.insert(i);
                }
            }
        }

        for i in self.entries.keys() {
            if !seen.contains(i) {
                list.push(i);
                seen.insert(i);
            }
        }

        list
    }

    pub fn index_unused(&self) -> BootIndex {
        let mut i = 0;
        loop {
            let idx = BootIndex(i);
            if !self.entries.contains_key(&idx) {
                return idx;
            }
            i += 1;
        }
    }

    pub fn index_title(&self, i: &BootIndex) -> String {
        let title: &str;

        let mut prop: Vec<&str> = Vec::new();
        if Some(*i) == self.current {
            prop.push("current")
        }
        if Some(*i) == self.next {
            prop.push("next")
        }

        if let Some(entry) = self.entries.get(i) {
            title = &entry.title;
            if entry.is_hidden() {
                prop.push("hidden")
            }
            if entry.is_app() {
                prop.push("app")
            }
        } else {
            title = "[missing]";
        };

        let pos;
        if let Some(o) = &self.order {
            if let Some(i) = o.position(i) {
                pos = format!("#{}", i + 1);
                prop.push(&pos);
            }
        }

        let prop_str = if !prop.is_empty() {
            format!("  [{}]", prop.join(","))
        } else {
            String::new()
        };

        format!("{}  {:32}{}", i, title, prop_str)
    }
}
