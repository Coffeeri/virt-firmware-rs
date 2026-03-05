//!
//! wrap classic native firmware image as igvm
//!
use clap::{CommandFactory, Parser};
use log::{error, info};
use std::fs;
use std::process::ExitCode;

use virtfw_igvm_tools::builder::Builder;
use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;
use virtfw_igvm_tools::x86regs::{flat32_mode_regs, real_mode_regs};

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-wrap",
          about = "wrap classic native firmware image as igvm",
          long_about = None)]
struct Args {
    /// enable native platform (default)
    #[arg(
        long,
        help_heading = "IGVM build options",
        conflicts_with = "no_native"
    )]
    native: bool,

    /// disable native platform
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "native")]
    no_native: bool,

    /// enable sev-snp platform
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "no_snp")]
    snp: bool,

    /// disable sev-snp platform (default)
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "snp")]
    no_snp: bool,

    /// generate parameters (default)
    #[arg(
        long,
        help_heading = "IGVM build options",
        conflicts_with = "no_params"
    )]
    params: bool,

    /// do not generate parameters
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "params")]
    no_params: bool,

    /// boot firmware in real mode (default)
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "flat32")]
    real16: bool,

    /// boot firmware in protected mode
    #[arg(long, help_heading = "IGVM build options", conflicts_with = "real16")]
    flat32: bool,

    /// print ovmf metadata
    #[arg(long, help_heading = "Print informations")]
    meta: bool,

    /// inspect generated igvm
    #[arg(long, help_heading = "Print informations")]
    inspect: bool,

    /// input firmware image
    #[arg(short, long, value_name = "CODE")]
    input: Option<String>,

    /// input uefi vars image
    #[arg(long, value_name = "VARS")]
    vars: Option<String>,

    /// output igvm image
    #[arg(short, long, value_name = "IGVM")]
    output: Option<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

impl Args {
    fn evaluate_bool(enable: bool, disable: bool, default: bool) -> bool {
        if enable {
            true
        } else if disable {
            false
        } else {
            default
        }
    }

    fn get_native(&self) -> bool {
        Args::evaluate_bool(self.native, self.no_native, true)
    }

    fn get_snp(&self) -> bool {
        Args::evaluate_bool(self.snp, self.no_snp, false)
    }

    fn get_params(&self) -> bool {
        Args::evaluate_bool(self.params, self.no_params, true)
    }

    fn get_flat32(&self) -> bool {
        Args::evaluate_bool(self.flat32, self.real16, false)
    }
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Debug)
        .init()
        .unwrap();

    if let Some(ref infile) = cfg.input {
        info!("reading {}", &infile);

        // code
        let firmware_res = fs::read(infile);
        if let Err(e) = firmware_res {
            error!("read {}: {}", &infile, e);
            return ExitCode::from(1);
        }
        let firmware = firmware_res.unwrap();

        let ovmfmeta = OvmfMeta::new(&firmware);
        if cfg.meta {
            if let Some(m) = ovmfmeta.as_ref() {
                m.print();
            }
        }

        // vars
        let uefivars = if let Some(ref varsfile) = cfg.vars {
            info!("reading {}", &varsfile);
            let uefivars_res = fs::read(varsfile);
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
        if cfg.get_native() {
            builder.add_native_platform();
            if cfg.get_flat32() {
                let regs = flat32_mode_regs(None);
                builder.add_native_context(&regs);
            } else {
                // nothing to do, this is the default
            }
        }

        // sev-snp
        if cfg.get_snp() {
            let regs = if cfg.get_flat32() {
                flat32_mode_regs(None)
            } else {
                real_mode_regs()
            };
            builder.add_snp_platform();
            builder.add_snp_vmsa_context(&regs, false);
            if let Some(m) = ovmfmeta.as_ref() {
                builder.add_ovmf_snp_pages(m);
            }
            builder.add_snp_policy(None);
        }

        // parameters
        if let Some(m) = ovmfmeta.as_ref() {
            if cfg.get_params() {
                builder.add_ovmf_igvm_params(m);
            }
        }

        // firmware code
        if !cfg.get_flat32() {
            builder.add_firmware_1m(&firmware);
        }
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

        if cfg.inspect {
            println!("IGVM image");
            inspect_igvm(&igvm);
        }

        if let Some(outfile) = cfg.output {
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
