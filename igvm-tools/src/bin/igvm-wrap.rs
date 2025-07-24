//!
//! wrap classic native firmware image as igvm
//!
use log::{error, info};
use std::cmp::min;
use std::env;
use std::fs;
use std::process::ExitCode;

use igvm::snp_defs::SevFeatures;
use igvm::{
    registers::X86Register, Arch, IgvmDirectiveHeader, IgvmFile, IgvmInitializationHeader,
    IgvmPlatformHeader, IgvmRevision,
};
use igvm_defs::{
    IgvmPageDataFlags, IgvmPageDataType, IgvmPlatformType, IGVM_VHS_SUPPORTED_PLATFORM,
    PAGE_SIZE_4K,
};

use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::ovmfmeta::{OvmfMeta, OvmfRegionType};
use virtfw_igvm_tools::x86regs::{real_mode_regs, vmsa_context};

pub const NATIVE_COMPAT: u32 = 1u32 << 0;
pub const SEV_SNP_COMPAT: u32 = 1u32 << 1;

struct Builder {
    revision: IgvmRevision,
    platforms: Vec<IgvmPlatformHeader>,
    initializations: Vec<IgvmInitializationHeader>,
    directives: Vec<IgvmDirectiveHeader>,
    compatibility_mask_all: u32,
}

impl Builder {
    fn new() -> Builder {
        let revision = IgvmRevision::V2 {
            arch: Arch::X64,
            page_size: PAGE_SIZE_4K.try_into().unwrap(),
        };
        let platforms = Vec::new();
        let initializations = Vec::new();
        let directives = Vec::new();
        Builder {
            revision,
            platforms,
            initializations,
            directives,
            compatibility_mask_all: 0,
        }
    }

    fn add_native_platform(&mut self) -> &mut Builder {
        let native = IgvmPlatformHeader::SupportedPlatform(IGVM_VHS_SUPPORTED_PLATFORM {
            compatibility_mask: NATIVE_COMPAT,
            highest_vtl: 2,
            platform_type: IgvmPlatformType::NATIVE,
            platform_version: 1,
            shared_gpa_boundary: 0,
        });
        self.compatibility_mask_all |= NATIVE_COMPAT;
        self.platforms.push(native);
        self
    }

    fn add_snp_platform(&mut self) -> &mut Builder {
        let snp = IgvmPlatformHeader::SupportedPlatform(IGVM_VHS_SUPPORTED_PLATFORM {
            compatibility_mask: SEV_SNP_COMPAT,
            highest_vtl: 2,
            platform_type: IgvmPlatformType::SEV_SNP,
            platform_version: 1,
            shared_gpa_boundary: 0,
        });
        self.compatibility_mask_all |= SEV_SNP_COMPAT;
        self.platforms.push(snp);
        self
    }

    fn add_snp_vmsa_context(&mut self, regs: &[X86Register]) -> &mut Builder {
        let features = SevFeatures::new().with_snp(true);
        let vmsa = vmsa_context(regs, features);
        let gpa = 0xFFFFFFFFF000;
        let ctx = IgvmDirectiveHeader::SnpVpContext {
            gpa,
            compatibility_mask: SEV_SNP_COMPAT,
            vp_index: 0,
            vmsa: Box::new(vmsa),
        };
        self.directives.push(ctx);
        self
    }

    fn add_empty_pages(
        &mut self,
        base: usize,
        size: usize,
        compatibility_mask: u32,
        data_type: IgvmPageDataType,
    ) -> &mut Builder {
        let pages = size / PAGE_SIZE_4K as usize;
        for pg in 0..pages {
            let start = pg * PAGE_SIZE_4K as usize;
            let page = IgvmDirectiveHeader::PageData {
                gpa: (base + start) as u64,
                compatibility_mask,
                flags: IgvmPageDataFlags::new(),
                data_type,
                data: vec![],
            };
            self.directives.push(page);
        }
        self
    }

    fn add_ovmf_snp_pages(&mut self, ovmfmeta: &OvmfMeta) -> &mut Builder {
        for r in &ovmfmeta.regions {
            let itype = match r.etype {
                OvmfRegionType::SevMemory => Some(IgvmPageDataType::NORMAL),
                OvmfRegionType::SevSecrets => Some(IgvmPageDataType::SECRETS),
                OvmfRegionType::SevCpuid => Some(IgvmPageDataType::CPUID_DATA),
                _ => None,
            };
            if let Some(t) = itype {
                self.add_empty_pages(r.memory.0, r.memory.1, SEV_SNP_COMPAT, t);
            }
        }
        self
    }

    fn add_data_pages(&mut self, base: usize, data: &[u8]) -> &mut Builder {
        let pages = data.len() / PAGE_SIZE_4K as usize;
        for pg in 0..pages {
            let start = pg * PAGE_SIZE_4K as usize;
            let end = start + PAGE_SIZE_4K as usize;
            let page = IgvmDirectiveHeader::PageData {
                gpa: (base + start) as u64,
                compatibility_mask: self.compatibility_mask_all,
                flags: IgvmPageDataFlags::new(),
                data_type: IgvmPageDataType::NORMAL,
                data: data[start..end].to_vec(),
            };
            self.directives.push(page);
        }
        self
    }

    fn add_firmware_4g(&mut self, firmware: &[u8]) -> &mut Builder {
        // all below 4GB
        let fwbase = (1u64 << 32) as usize - firmware.len();
        self.add_data_pages(fwbase, firmware);
        self
    }

    fn add_firmware_1m(&mut self, firmware: &[u8]) -> &mut Builder {
        // 128k below 1MB
        let lowsize = min(128 * 1024, firmware.len());
        let lowbase = (1u64 << 20) as usize - lowsize;
        let offset = firmware.len() - lowsize;
        self.add_data_pages(lowbase, &firmware[offset..]);
        self
    }

    fn add_uefivars(&mut self, uefivars: &[u8], fwsize: usize) -> &mut Builder {
        // varstore (below 4G + firmware code)
        let varsbase = (1u64 << 32) as usize - uefivars.len() - fwsize;
        self.add_data_pages(varsbase, uefivars);
        self
    }

    fn finalize(&self) -> Result<IgvmFile, igvm::Error> {
        IgvmFile::new(
            self.revision,
            self.platforms.clone(),
            self.initializations.clone(),
            self.directives.clone(),
        )
    }
}

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

fn print_ovmfmeta(ovmfmeta: &OvmfMeta) {
    println!("OVMF regions");
    for r in &ovmfmeta.regions {
        println!("  {r}");
    }
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
                print_ovmfmeta(m)
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
