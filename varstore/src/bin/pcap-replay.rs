///
/// test utility: dump pcap files
///
use clap::Parser;
use log::{error, info};
use std::process::ExitCode;
use uguid::Guid;

use virtfw_libefi::guids;
use virtfw_varstore::json::JsonEfiVarStore;
use virtfw_varstore::mm::core::MmCore;
use virtfw_varstore::mm::util;
use virtfw_varstore::mm::variable::{MmVariableHeader, MmVariableRequest};
use virtfw_varstore::pcap::parse::{VarstorePcapItem, VarstorePcapParser};
use virtfw_varstore::pcap::replay::replay_pcap;
use virtfw_varstore::store::EfiVarStore;

#[derive(Parser, Debug)]
#[command(version, name = "pcap-replay", about = "dump varstore pcap files", long_about = None)]
struct Args {
    /// turn on debug logging
    #[arg(short, long)]
    debug: bool,

    /// print pcap file
    #[arg(short, long)]
    print: bool,

    /// replay pcap file
    #[arg(long)]
    replay: bool,

    /// pcap file (print or replay)
    #[arg(long, value_name = "PCAP")]
    pcap: Option<String>,

    /// json file (initial replay state)
    #[arg(long, value_name = "JSON")]
    json: Option<String>,
}

fn print_request(data: &[u8]) {
    let core = MmCore::new(data).unwrap();
    let guid = Guid::from_bytes(core.guid);
    match guid {
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
    let guid = Guid::from_bytes(core.guid);
    match guid {
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

fn print_pcap(filename: &str) -> std::io::Result<()> {
    let parser = VarstorePcapParser::new_file(filename)?;

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

fn main() -> ExitCode {
    let cfg = Args::parse();

    let loglevel = if cfg.debug {
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

    if let Some(pcap) = cfg.pcap {
        if cfg.print {
            let res = print_pcap(&pcap);
            if let Err(e) = res {
                error!("{e}");
            } else {
                info!("ok");
            }
        }

        if cfg.replay {
            let mut store = EfiVarStore::new();

            if let Some(filename) = cfg.json {
                let Ok(content) = std::fs::read_to_string(filename) else {
                    error!("read json file");
                    return ExitCode::from(1);
                };
                let Ok(jstore) = serde_json::from_str::<JsonEfiVarStore>(&content) else {
                    error!("parse json file");
                    return ExitCode::from(1);
                };
                store.json_import(&jstore);
            }

            let parser = VarstorePcapParser::new_file(&pcap).expect("open pcap file");
            replay_pcap(parser, &mut store);
        }
    }

    ExitCode::from(0)
}
