//! read/write efi variables from linux efivarfs

extern crate std;
use std::format;
use std::fs;
use std::io;
use std::io::Write;
use std::string::{String, ToString};
use std::vec::Vec;

use log::debug;
use log::trace;
use uguid::Guid;
use zerocopy::FromBytes;

use crate::efivar::types;

fn varstore_filename(name: &str, guid: &Guid) -> String {
    let filename = format!("/sys/firmware/efi/efivars/{name}-{guid}");
    filename
}

pub fn varstore_ls() -> io::Result<Vec<types::EfiVarId>> {
    let mut vars = Vec::new();

    let dir = fs::read_dir("/sys/firmware/efi/efivars")?;
    for e in dir {
        if let Ok(filename) = e?.file_name().into_string() {
            if filename.len() > 37 {
                let namelen = filename.len() - 37;
                let name = filename.get(..namelen).unwrap();
                let guid_str = filename.get((namelen + 1)..).unwrap();
                let guid = Guid::try_parse(guid_str).unwrap();
                let id = types::EfiVarId::new(name.to_string(), guid);
                vars.push(id);
            }
        }
    }

    Ok(vars)
}

pub fn varstore_read_id(id: &types::EfiVarId) -> Option<types::EfiVar> {
    let filename = varstore_filename(id.name(), id.guid());
    debug!("reading {filename}");
    let Ok(blob) = fs::read(filename) else {
        return None;
    };
    let Ok((attr, data)) = u32::read_from_prefix(&blob) else {
        return None;
    };
    Some(types::EfiVar::new_from_slice(
        id.clone(),
        types::EfiVarAttr::from(attr),
        data,
    ))
}

pub fn varstore_read(name: &str, guid: &Guid) -> Option<types::EfiVar> {
    let id = types::EfiVarId::new(name.into(), *guid);
    varstore_read_id(&id)
}

pub fn varstore_write(var: &types::EfiVar) -> io::Result<()> {
    // must write variable with a single syscall
    // so go prepare a buffer with all data
    let mut blob: Vec<u8> = Vec::new();
    let attr: u32 = var.attr().into();
    blob.extend_from_slice(&attr.to_le_bytes());
    blob.extend_from_slice(var.data());
    trace!("datablob={blob:?}");

    let filename = varstore_filename(var.name(), var.guid());
    debug!("writing {filename}");
    let mut f = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(filename)?;
    f.write_all(&blob)?;

    Ok(())
}

pub fn varstore_delete(name: &str, guid: &Guid) -> io::Result<()> {
    let filename = varstore_filename(name, guid);
    debug!("deleting {filename}");
    fs::remove_file(filename)?;
    Ok(())
}

pub fn varstore_delete_id(id: &types::EfiVarId) -> io::Result<()> {
    varstore_delete(id.name(), id.guid())
}
