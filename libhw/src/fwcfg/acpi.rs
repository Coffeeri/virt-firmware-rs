/// parse acpi table loader commands from qemu
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::addr_of;
use log::{trace, warn};
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
    align: u32,
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
        align: u32,
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
    data: &'static mut [u8],
    addr: usize,
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
                    LoaderCommand::Alloc {
                        filename: bytes_to_string(&a.filename),
                        align: a.align,
                        zone: a.zone,
                    }
                }
                CMD_ADD_POINTER => {
                    let (p, _) = LoaderAddPtr::read_from_prefix(r).unwrap();
                    LoaderCommand::AddPtr {
                        pointer: bytes_to_string(&p.pointer),
                        pointee: bytes_to_string(&p.pointee),
                        offset: p.offset,
                        size: p.size,
                    }
                }
                CMD_ADD_CHECKSUM => {
                    let (c, _) = LoaderChecksum::read_from_prefix(r).unwrap();
                    LoaderCommand::Checksum {
                        filename: bytes_to_string(&c.filename),
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

            trace!("{:?}", command);
            self.commands.push(command);
        }
        Some(())
    }

    pub fn load_tables<A>(&mut self, alloc_func: A) -> Result<(), &str>
    where
        A: Fn(usize, usize) -> Option<&'static mut [u8]>,
    {
        for c in &self.commands {
            if let LoaderCommand::Alloc {
                filename, align, ..
            } = c
            {
                let (item, size) = self
                    .fwcfg
                    .findfile(filename)
                    .ok_or("fwcfg file not found")?;

                let data_opt = alloc_func(size as usize, *align as usize);
                let data = data_opt.ok_or("allocation failed")?;
                self.fwcfg
                    .read_dma(Some(item), data.as_mut_ptr().cast(), size as usize);

                let addr = addr_of!(*data).addr();
                let mem = LoaderMemory {
                    filename: filename.clone(),
                    item,
                    size,
                    data,
                    addr,
                };

                trace!(
                    "LoaderMemory: {}, item {}, size {}, addr {:x}",
                    mem.filename,
                    mem.item,
                    mem.size,
                    mem.addr
                );
                self.memories.push(mem);
            }
        }
        Ok(())
    }

    pub fn run_commands(&mut self) -> Result<(), &str> {
        for c in &self.commands {
            match c {
                LoaderCommand::AddPtr {
                    pointer,
                    pointee,
                    offset,
                    size,
                } => {
                    let addr = self
                        .memories
                        .iter()
                        .find(|m| &m.filename == pointee)
                        .ok_or("allocation not found")?
                        .addr;
                    let src: &mut LoaderMemory = self
                        .memories
                        .iter_mut()
                        .find(|m| &m.filename == pointer)
                        .ok_or("allocation not found")?;
                    let data =
                        &mut src.data[(*offset as usize)..(*offset as usize + *size as usize)];
                    match size {
                        4 => {
                            let array: [u8; 4] = data.try_into().unwrap();
                            let mut value = <u32>::from_le_bytes(array);
                            value += addr as u32;
                            data.copy_from_slice(&value.to_le_bytes());
                        }
                        8 => {
                            let array: [u8; 8] = data.try_into().unwrap();
                            let mut value = <u64>::from_le_bytes(array);
                            value += addr as u64;
                            data.copy_from_slice(&value.to_le_bytes());
                        }
                        _ => {
                            return Err("unhandled ptr size");
                        }
                    }
                }
                LoaderCommand::Checksum {
                    filename,
                    offset,
                    start,
                    length,
                } => {
                    let mem: &mut LoaderMemory = self
                        .memories
                        .iter_mut()
                        .find(|m| &m.filename == filename)
                        .ok_or("allocation not found")?;
                    let data = &mem.data[(*start as usize)..((start + length) as usize)];
                    let sum: u32 = data.iter().map(|b| *b as u32).sum();
                    let fixup = (0u32.wrapping_sub(sum) & 0xff) as u8;
                    mem.data[*offset as usize] = fixup;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn rsdp(&self) -> Option<*const ()> {
        let mem = self
            .memories
            .iter()
            .find(|m| &m.filename == "etc/acpi/rsdp")?;
        Some(mem.data.as_ptr().cast())
    }
}
