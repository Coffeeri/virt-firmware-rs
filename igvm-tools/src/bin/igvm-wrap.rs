//!
//! wrap classic native firmware image as igvm
//!
use log::{error, info};
use std::env;
use std::fs;
use std::process::ExitCode;

use virtfw_igvm_tools::builder::Builder;
use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;
use virtfw_igvm_tools::x86regs::real_mode_regs;

fn parse_args() -> Option<getopts::Matches> {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optflag("h", "help", "print this help text");
    opts.optflag("", "meta", "print ovmf metadata");
    opts.optflag("", "inspect", "inspect generated igvm");

    opts.optflag("", "no-native", "disable native platform");
    opts.optflag("", "snp", "enable sev-snp platform");

    opts.optopt("i", "input", "input firmware image", "CODE");
    opts.optopt("", "vars", "input uefi vars image", "VARS");
    opts.optopt("o", "output", "output igvm image", "IGVM");

    let cfg_res = opts.parse(&args[1..]);
    let Ok(cfg) = cfg_res else {
        println!("{:?}", cfg_res.err());
        return None;
    };
    if cfg.opt_present("help") {
        print!(
            "{}",
            opts.usage("igvm-wrap -- wrap classic native firmware image as igvm")
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
        .verbosity(stderrlog::LogLevelNum::Debug)
        .init()
        .unwrap();

    if let Some(infile) = cfg.opt_str("input") {
        info!("reading {}", &infile);

        // code
        let firmware_res = fs::read(&infile);
        if let Err(e) = firmware_res {
            error!("read {}: {}", &infile, e);
            return ExitCode::from(1);
        }
        let firmware = firmware_res.unwrap();

        let ovmfmeta = OvmfMeta::new(&firmware);
        if cfg.opt_present("meta") {
            if let Some(m) = ovmfmeta.as_ref() {
                m.print();
            }
        }

        // vars
        let uefivars = if let Some(varsfile) = cfg.opt_str("vars") {
            info!("reading {}", &varsfile);
            let uefivars_res = fs::read(&varsfile);
            if let Err(e) = uefivars_res {
                error!("read {}: {}", &varsfile, e);
                return ExitCode::from(1);
            }
            Some(uefivars_res.unwrap())
        } else {
            None
        };

        let mut builder = Builder::new();

        // native
        if !cfg.opt_present("no-native") {
            builder.add_native_platform();
        }

        // sev-snp
        if cfg.opt_present("snp") {
            builder.add_snp_platform();
            builder.add_snp_vmsa_context(&real_mode_regs());
            if let Some(m) = ovmfmeta.as_ref() {
                builder.add_ovmf_snp_pages(m);
            }
        }

        // firmware code
        builder.add_firmware_1m(&firmware);
        builder.add_firmware_4g(&firmware);

        // uefi vars
        if let Some(v) = uefivars {
            builder.add_uefivars(&v, firmware.len());
        }

        let igvm_res = builder.finalize();
        if let Err(e) = igvm_res {
            error!("build igvm: {e}");
            return ExitCode::from(1);
        }
        let igvm = igvm_res.unwrap();

        if cfg.opt_present("inspect") {
            println!("IGVM image");
            inspect_igvm(&igvm);
        }

        if let Some(outfile) = cfg.opt_str("output") {
            let mut blob = Vec::new();
            let res = igvm.serialize(&mut blob);
            if let Err(e) = res {
                error!("serialize igvm: {e}");
                return ExitCode::from(1);
            }

            info!("writing {}", &outfile);
            let res = fs::write(&outfile, blob);
            if let Err(e) = res {
                error!("write {}: {}", &outfile, e);
                return ExitCode::from(1);
            }
        } else {
            info!("igvm ok");
        }
    }

    ExitCode::from(0)
}
