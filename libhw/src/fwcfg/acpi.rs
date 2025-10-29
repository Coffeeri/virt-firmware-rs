/// parse acpi table loader commands from qemu
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use log::{debug, warn};
use zerocopy::FromBytes;

use crate::fwcfg::{bytes_to_string, FwCfg, FILENAME_LEN};

const CMD_ALLOCATE: u32 = 1;
const CMD_ADD_POINTER: u32 = 2;
const CMD_ADD_CHECKSUM: u32 = 3;
// const WritePointer: u32 = 4;

#[derive(FromBytes)]
#[repr(C)]
struct LoaderCmdType {
    cmd: u32,
}

#[derive(FromBytes)]
#[repr(C)]
struct LoaderAlloc {
    filename: [u8; FILENAME_LEN],
    alignment: u32,
    zone: u8,
}

#[derive(FromBytes)]
#[repr(C)]
struct LoaderAddPtr {
    pointer: [u8; FILENAME_LEN],
    pointee: [u8; FILENAME_LEN],
    offset: u32,
    size: u8,
}

#[derive(FromBytes)]
#[repr(C)]
struct LoaderChecksum {
    filename: [u8; FILENAME_LEN],
    offset: u32,
    start: u32,
    length: u32,
}

#[derive(Debug)]
pub enum LoaderCommand {
    Alloc {
        filename: String,
        alignment: u32,
        zone: u8,
    },
    AddPtr {
        pointer: String,
        pointee: String,
        offset: u32,
        size: u8,
    },
    Checksum {
        filename: String,
        offset: u32,
        start: u32,
        length: u32,
    },
}

#[derive(Debug)]
pub struct LoaderMemory {
    filename: String,
    item: u16,
    size: u32,
}

pub struct AcpiLoader<'f> {
    fwcfg: &'f dyn FwCfg,
    commands: Vec<LoaderCommand>,
    memories: Vec<LoaderMemory>,
}

impl<'f> AcpiLoader<'f> {
    pub fn new(fwcfg: &'f dyn FwCfg) -> Option<Self> {
        let mut loader = Self {
            fwcfg,
            commands: Vec::new(),
            memories: Vec::new(),
        };
        loader.get_commands()?;
        Some(loader)
    }

    fn get_commands(&mut self) -> Option<()> {
        let loader = self.fwcfg.load_file("etc/table-loader")?;

        for entry in loader.chunks(128) {
            let (cmdtype, r) = LoaderCmdType::read_from_prefix(entry).unwrap();
            let command = match cmdtype.cmd {
                0 => {
                    // end of list
                    break;
                }
                CMD_ALLOCATE => {
                    let (a, _) = LoaderAlloc::read_from_prefix(r).unwrap();
                    let filename = bytes_to_string(&a.filename);

                    LoaderCommand::Alloc {
                        filename,
                        alignment: a.alignment,
                        zone: a.zone,
                    }
                }
                CMD_ADD_POINTER => {
                    let (p, _) = LoaderAddPtr::read_from_prefix(r).unwrap();
                    let ptr = bytes_to_string(&p.pointer);
                    let ptee = bytes_to_string(&p.pointee);
                    LoaderCommand::AddPtr {
                        pointer: ptr,
                        pointee: ptee,
                        offset: p.offset,
                        size: p.size,
                    }
                }
                CMD_ADD_CHECKSUM => {
                    let (c, _) = LoaderChecksum::read_from_prefix(r).unwrap();
                    let filename = bytes_to_string(&c.filename);
                    LoaderCommand::Checksum {
                        filename,
                        offset: c.offset,
                        start: c.start,
                        length: c.length,
                    }
                }
                _ => {
                    warn!("unknown loader cmd: {}", cmdtype.cmd);
                    break;
                }
            };

            debug!("{:?}", &command);
            self.commands.push(command);
        }
        Some(())
    }

    pub fn load_tables(&mut self) {
        for c in &self.commands {
            if let LoaderCommand::Alloc { filename, .. } = c {
                let Some((item, size)) = self.fwcfg.findfile(filename) else {
                    warn!("fwcfg file not found: {}", filename);
                    break;
                };

                let mem = LoaderMemory {
                    filename: filename.clone(),
                    item,
                    size,
                };
                debug!(
                    "LoaderMemory: {}, item {}, size {}",
                    &mem.filename, mem.item, mem.size
                );
                self.memories.push(mem);
            }
        }
    }
}
