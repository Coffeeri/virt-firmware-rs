use std::fs::File;
use std::process::ExitCode;

use log::{error, info};

use virtfw_libefi::guids;
use virtfw_varstore::mm::core::MmCore;
use virtfw_varstore::mm::variable::MmVariableRequest;
use virtfw_varstore::pcap::parse::{VarstorePcapItem, VarstorePcapParser};

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optflag("d", "debug", "turn on debug logging");
    opts.optopt("", "pcap", "pcap file to parse", "PCAP");

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

fn print_request(data: &[u8]) {
    let core = MmCore::new(data).unwrap();
    match core.guid {
        guids::EfiSmmVariableProtocol => {
            let var_res = MmVariableRequest::new(core.data);
            if let Err(e) = var_res.as_ref() {
                error!("{e}");
            }
            println!("request/var: {}", var_res.unwrap());
        }
        _ => {
            println!("request/core: {}", core);
        }
    }
}

fn print_pcap(filename: &str) -> std::io::Result<()> {
    let file = File::open(filename)?;
    let parser = VarstorePcapParser::new(&file)?;
    for p in parser {
        match p {
            VarstorePcapItem::Request(d) => {
                print_request(&d);
            }
            VarstorePcapItem::Reply(_) => {
                // ignore for now
            }
            _ => {
                println!("{}", p);
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let Some(cfg) = parse_args() else {
        return ExitCode::from(1);
    };
    loginit(&cfg);

    if let Some(pcap) = cfg.opt_str("pcap") {
        let res = print_pcap(&pcap);
        if let Err(e) = res {
            error!("{e}");
        } else {
            info!("ok");
        }
    }

    ExitCode::from(0)
}
