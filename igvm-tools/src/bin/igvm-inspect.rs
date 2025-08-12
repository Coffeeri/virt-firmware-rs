use log::info;
use std::env;
use std::process::ExitCode;

use igvm::{IgvmDirectiveHeader, IgvmFile};
use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;

fn resetvector(d: &IgvmDirectiveHeader) -> Option<&[u8]> {
    match d {
        IgvmDirectiveHeader::PageData { gpa, data, .. } => {
            if *gpa == 0xfffff000 {
                Some(data)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optflag("h", "help", "print this help text");
    opts.optflag("", "meta", "print ovmf metadata");

    let cfg_res = opts.parse(&args[1..]);
    let Ok(cfg) = cfg_res else {
        println!("{:?}", cfg_res.err());
        return None;
    };
    if cfg.opt_present("help") {
        print!(
            "{}",
            opts.usage("igvm-inspect -- inspect igvm file content")
        );
        return None;
    };

    Some(cfg)
}

fn main() -> ExitCode {
    let Some(cfg) = parse_args() else {
        return ExitCode::from(1);
    };

    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();

    for arg in &cfg.free {
        info!("{arg}");

        let blob = std::fs::read(arg).expect("file read error");
        let igvm = IgvmFile::new_from_binary(&blob, None).expect("file parse error");

        inspect_igvm(&igvm);

        if cfg.opt_present("meta") {
            for page in igvm.directives().iter().filter_map(resetvector) {
                if let Some(ovmfmeta) = OvmfMeta::new(page) {
                    ovmfmeta.print();
                }
            }
        }
    }

    ExitCode::from(0)
}
