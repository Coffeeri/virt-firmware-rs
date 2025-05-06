#![cfg(feature = "std")]

extern crate std;
use std::boxed::Box;
use std::io::{Error, ErrorKind, Read};
use std::vec;
use std::vec::Vec;
use zerocopy::FromBytes;

use crate::pcap::types::{PcapFileHeader, PcapPacketHeader, PCAP_MAGIC};

#[derive(Debug)]
pub enum VarstorePcapItem {
    Reset,
    Request(Vec<u8>),
    Reply(Vec<u8>),
}

impl std::fmt::Display for VarstorePcapItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VarstorePcapItem::Reset => {
                write!(f, "pcap/reset")
            }
            VarstorePcapItem::Request(d) => {
                write!(f, "pcap/req: {} bytes", d.len())
            }
            VarstorePcapItem::Reply(d) => {
                write!(f, "pcap/res: {} bytes", d.len())
            }
        }
    }
}

pub struct VarstorePcapParser {
    reader: Box<dyn Read>,
}

impl VarstorePcapParser {
    pub fn new(mut reader: Box<dyn Read>) -> Result<Self, std::io::Error> {
        let fhdr = PcapFileHeader::read_from_io(&mut *reader)?;
        if fhdr.magic != PCAP_MAGIC {
            return Err(Error::new(ErrorKind::Other, "magic mismatch"));
        }
        if fhdr.network != 147 {
            return Err(Error::new(ErrorKind::Other, "linktype mismatch"));
        }
        let parser = VarstorePcapParser { reader };
        Ok(parser)
    }
}

impl Iterator for VarstorePcapParser {
    type Item = VarstorePcapItem;
    fn next(&mut self) -> Option<VarstorePcapItem> {
        let phdr = PcapPacketHeader::read_from_io(&mut *self.reader).ok()?;
        let ptype = u32::read_from_io(&mut *self.reader).ok()?;
        let mut blob: Vec<u8> = vec![0; phdr.incl_len as usize - 4];
        self.reader.read_exact(&mut blob).ok()?;
        let item = match ptype {
            1 => VarstorePcapItem::Reset,
            2 => VarstorePcapItem::Request(blob),
            3 => VarstorePcapItem::Reply(blob),
            _ => {
                return None;
            }
        };
        Some(item)
    }
}
