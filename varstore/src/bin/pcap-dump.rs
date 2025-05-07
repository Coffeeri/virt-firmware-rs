///
/// test utility: dump pcap files
///
use flate2::read::GzDecoder;
use std::fs::File;
use std::process::ExitCode;

use log::{debug, error, info};

use virtfw_libefi::guids;
use virtfw_varstore::mm::core::{core_request, MmCore};
use virtfw_varstore::mm::util;
use virtfw_varstore::mm::variable::{MmVariableHeader, MmVariableRequest};
use virtfw_varstore::pcap::parse::{VarstorePcapItem, VarstorePcapParser};
use virtfw_varstore::store::EfiVarStore;

fn print_request(data: &[u8]) {
    let core = MmCore::new(data).unwrap();
    match core.guid {
        guids::EfiSmmVariableProtocol => {
            let var_res = MmVariableRequest::new(core.data);
            if let Err(e) = var_res.as_ref() {
                error!("variable/req/parse: {e}");
                return;
            }
            let parsed = var_res.unwrap().parsed;
            if parsed.is_none() {
                error!("variable/req/parse: error");
                return;
            }
            println!("variable/req: {}", parsed.unwrap());
        }
        _ => {
            println!("core/req: {}", core);
        }
    }
}

fn print_reply(data: &[u8]) {
    let core = MmCore::new(data).unwrap();
    match core.guid {
        guids::EfiSmmVariableProtocol => {
            let hdr_res = MmVariableHeader::new(core.data);
            if let Err(e) = hdr_res.as_ref() {
                error!("{e}");
            };
            let status = util::u64_to_status(hdr_res.unwrap().0.status);
            println!("variable/rsp: {}", status);
        }
        _ => {
            println!("core/rsp: {}", core);
        }
    }
}

fn open_pcap(filename: &str) -> std::io::Result<VarstorePcapParser> {
    let file = File::open(filename)?;

    let parser = if filename.ends_with(".gz") {
        let ungz = GzDecoder::new(file);
        VarstorePcapParser::new(Box::new(ungz))?
    } else {
        VarstorePcapParser::new(Box::new(file))?
    };

    Ok(parser)
}

fn print_pcap(filename: &str) -> std::io::Result<()> {
    let parser = open_pcap(filename)?;

    for p in parser {
        match p {
            VarstorePcapItem::Request(d) => {
                print_request(&d);
            }
            VarstorePcapItem::Reply(d) => {
                print_reply(&d);
            }
            _ => {
                println!("{}", p);
            }
        }
    }
    Ok(())
}

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

fn process_pcap(filename: &str) -> std::io::Result<()> {
    let mut store = EfiVarStore::new();
    let parser = open_pcap(filename)?;

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
    Ok(())
}

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optflag("d", "debug", "turn on debug logging");
    opts.optflag("p", "print", "print pcap file");
    opts.optflag("", "run", "print pcap file");
    opts.optopt("", "pcap", "pcap file", "PCAP");

    let cfg_res = opts.parse(&args[1..]);
    let Ok(cfg) = cfg_res else {
        println!("{:?}", cfg_res.err());
        return None;
    };
    if cfg.opt_present("help") {
        print!("{}", opts.usage("dump varstore pcap files"));
        return None;
    };

    Some(cfg)
}

fn loginit(cfg: &getopts::Matches) {
    let loglevel = if cfg.opt_present("debug") {
        stderrlog::LogLevelNum::Debug
    } else {
        stderrlog::LogLevelNum::Info
    };
    stderrlog::new()
        .module(module_path!())
        .module("virtfw_libefi")
        .module("virtfw_varstore")
        .verbosity(loglevel)
        .init()
        .unwrap();
}

fn main() -> ExitCode {
    let Some(cfg) = parse_args() else {
        return ExitCode::from(1);
    };
    loginit(&cfg);

    if let Some(pcap) = cfg.opt_str("pcap") {
        if cfg.opt_present("print") {
            let res = print_pcap(&pcap);
            if let Err(e) = res {
                error!("{e}");
            } else {
                info!("ok");
            }
        }
        if cfg.opt_present("run") {
            let res = process_pcap(&pcap);
            if let Err(e) = res {
                error!("{e}");
            } else {
                info!("ok");
            }
        }
    }

    ExitCode::from(0)
}
