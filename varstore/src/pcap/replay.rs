#![cfg(feature = "std")]

extern crate std;
use alloc::vec::Vec;

use log::{debug, error, info};

use crate::mm::core::core_request;
use crate::pcap::parse::{VarstorePcapItem, VarstorePcapParser};
use crate::store::EfiVarStore;

fn may_fail(rsp: &[u8]) -> bool {
    // QUERY_VARIABLE_INFO
    if rsp.starts_with(&[
        0x33, 0xd5, 0x32, 0xed, 0xe6, 0x99, 0x09, 0x42, 0x9c, 0xc0, 0x2d, 0x72, 0xcd, 0xd9, 0x98,
        0xa7, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ]) {
        return true;
    }

    false
}

fn pchar(byte: u8) -> char {
    if byte.is_ascii() && byte > 0x20 {
        byte as char
    } else {
        '.'
    }
}

fn diff_buffer(actual: &[u8], expect: &[u8]) {
    let count = std::cmp::min(actual.len(), expect.len());
    for i in 0..count {
        if actual[i] != expect[i] {
            error!(
                "  offset 0x{:x}: actual 0x{:02x} {} (expected 0x{:02x} {})",
                i,
                actual[i],
                pchar(actual[i]),
                expect[i],
                pchar(expect[i]),
            );
        } else {
            debug!(
                "  offset 0x{:x}: actual 0x{:02x} {}",
                i,
                actual[i],
                pchar(actual[i]),
            );
        }
    }
}

pub fn replay_pcap(parser: VarstorePcapParser) -> EfiVarStore {
    let mut store = EfiVarStore::new();

    let mut rsp = Vec::new();
    for p in parser {
        match p {
            VarstorePcapItem::Request(d) => {
                rsp = d.clone();
                core_request(&mut store, &mut rsp);
            }
            VarstorePcapItem::Reply(d) => {
                if rsp != d {
                    if may_fail(&rsp) {
                        info!("reply mismatch (may fail)");
                    } else {
                        error!("reply mismatch");
                        diff_buffer(&rsp, &d);
                        panic!();
                    }
                }
            }
            VarstorePcapItem::Reset => {
                debug!("reset");
                store.reset();
            }
        }
    }

    store
}
