///
/// test utility: dump pcap files
///
use std::process::ExitCode;

use log::{error, info};

use virtfw_libefi::guids;
use virtfw_varstore::json::JsonEfiVarStore;
use virtfw_varstore::mm::core::MmCore;
use virtfw_varstore::mm::util;
use virtfw_varstore::mm::variable::{MmVariableHeader, MmVariableRequest};
use virtfw_varstore::pcap::parse::{VarstorePcapItem, VarstorePcapParser};
use virtfw_varstore::pcap::replay::replay_pcap;
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

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optflag("d", "debug", "turn on debug logging");
    opts.optflag("p", "print", "print pcap file");
    opts.optflag("", "replay", "replay pcap file");
    opts.optopt("", "pcap", "pcap file (print or replay)", "PCAP");
    opts.optopt("", "json", "json file (initial replay state)", "JSON");

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

        if cfg.opt_present("replay") {
            let mut store = EfiVarStore::new();

            if let Some(filename) = cfg.opt_str("json") {
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
