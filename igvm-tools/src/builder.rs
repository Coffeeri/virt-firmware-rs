use std::cmp::{min, Ordering};
use std::collections::HashSet;

use igvm::snp_defs::SevFeatures;
use igvm::{
    registers::X86Register, Arch, IgvmDirectiveHeader, IgvmFile, IgvmInitializationHeader,
    IgvmPlatformHeader, IgvmRevision,
};
use igvm_defs::{
    IgvmPageDataFlags, IgvmPageDataType, IgvmPlatformType, SnpPolicy, IGVM_VHS_PARAMETER,
    IGVM_VHS_PARAMETER_INSERT, IGVM_VHS_SUPPORTED_PLATFORM, PAGE_SIZE_4K,
};

use crate::ovmfmeta::{OvmfMeta, OvmfRegionType};
use crate::x86regs::{native_context, vmsa_context};

pub const NATIVE_COMPAT: u32 = 1u32 << 0;
pub const SEV_SNP_COMPAT: u32 = 1u32 << 1;

pub struct Builder {
    revision: IgvmRevision,
    platforms: Vec<IgvmPlatformHeader>,
    initializations: Vec<IgvmInitializationHeader>,
    directives: Vec<IgvmDirectiveHeader>,
    compatibility_mask_all: u32,
}

impl Builder {
    fn revision() -> IgvmRevision {
        IgvmRevision::V2 {
            arch: Arch::X64,
            page_size: PAGE_SIZE_4K.try_into().unwrap(),
        }
    }

    pub fn new() -> Builder {
        let platforms = Vec::new();
        let initializations = Vec::new();
        let directives = Vec::new();
        Builder {
            revision: Self::revision(),
            platforms,
            initializations,
            directives,
            compatibility_mask_all: 0,
        }
    }

    pub fn add_native_platform(&mut self) -> &mut Builder {
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

    pub fn add_native_context(&mut self, regs: &[X86Register]) -> &mut Builder {
        let native = native_context(regs);
        let ctx = IgvmDirectiveHeader::X64NativeVpContext {
            compatibility_mask: NATIVE_COMPAT,
            vp_index: 0,
            context: Box::new(native),
        };
        self.directives.push(ctx);
        self
    }

    pub fn add_snp_platform(&mut self) -> &mut Builder {
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

    pub fn add_snp_vmsa_context(&mut self, regs: &[X86Register], debug: bool) -> &mut Builder {
        let features = SevFeatures::new().with_snp(true).with_debug_swap(debug);
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

    pub fn add_snp_policy(&mut self, snp_policy: Option<SnpPolicy>) -> &mut Builder {
        let default_policy = SnpPolicy::new().with_reserved_must_be_one(1).with_smt(1);
        let policy = IgvmInitializationHeader::GuestPolicy {
            policy: snp_policy.unwrap_or(default_policy).into(),
            compatibility_mask: SEV_SNP_COMPAT,
        };
        self.initializations.push(policy);
        self
    }

    pub fn add_empty_pages(
        &mut self,
        base: usize,
        size: usize,
        compatibility_mask: u32,
        data_type: IgvmPageDataType,
    ) -> &mut Builder {
        let psize = PAGE_SIZE_4K as usize;
        let pages = size.div_ceil(psize);
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

    pub fn add_empty_normal_pages(&mut self, base: usize, size: usize) -> &mut Builder {
        self.add_empty_pages(
            base,
            size,
            self.compatibility_mask_all,
            IgvmPageDataType::NORMAL,
        )
    }

    pub fn add_ovmf_snp_pages(&mut self, ovmfmeta: &OvmfMeta) -> &mut Builder {
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

    pub fn add_igvm_param_area(&mut self, index: u32, bytes: u64) {
        self.directives.push(IgvmDirectiveHeader::ParameterArea {
            parameter_area_index: index,
            number_of_bytes: bytes,
            initial_data: Vec::new(),
        });
    }

    fn add_igvm_param<P>(&mut self, mkdirective: P, index: u32, offset: u32)
    where
        P: FnOnce(IGVM_VHS_PARAMETER) -> IgvmDirectiveHeader,
    {
        let param = IGVM_VHS_PARAMETER {
            parameter_area_index: index,
            byte_offset: offset,
        };
        self.directives.push(mkdirective(param));
    }

    pub fn add_igvm_param_memmap(&mut self, index: u32, offset: u32) {
        self.add_igvm_param(IgvmDirectiveHeader::MemoryMap, index, offset);
    }

    pub fn add_igvm_param_vpcount(&mut self, index: u32, offset: u32) {
        self.add_igvm_param(IgvmDirectiveHeader::VpCount, index, offset);
    }

    pub fn add_igvm_param_insert(&mut self, index: u32, gpa: u64) {
        self.directives.push(IgvmDirectiveHeader::ParameterInsert(
            IGVM_VHS_PARAMETER_INSERT {
                parameter_area_index: index,
                gpa,
                compatibility_mask: self.compatibility_mask_all,
            },
        ));
    }

    pub fn add_ovmf_igvm_params(&mut self, ovmfmeta: &OvmfMeta) -> &mut Builder {
        let Some(area) = &ovmfmeta
            .regions
            .iter()
            .find(|r| r.etype == OvmfRegionType::IgvmParamArea)
        else {
            return self;
        };

        self.add_igvm_param_area(0, area.memory.1 as u64);
        for r in &ovmfmeta.regions {
            match r.etype {
                OvmfRegionType::IgvmParamMemoryMap => {
                    self.add_igvm_param_memmap(0, r.memory.0 as u32);
                }
                OvmfRegionType::IgvmParamVpCount => {
                    self.add_igvm_param_vpcount(0, r.memory.0 as u32);
                }
                _ => {}
            }
        }
        self.add_igvm_param_insert(0, area.memory.0 as u64);

        self
    }

    pub fn add_data_pages_flags(
        &mut self,
        base: usize,
        data: &[u8],
        flags: IgvmPageDataFlags,
    ) -> &mut Builder {
        let psize = PAGE_SIZE_4K as usize;
        let pages = data.len().div_ceil(psize);
        for pg in 0..pages {
            let start = pg * psize;
            let end = min(start + psize, data.len());
            let page = IgvmDirectiveHeader::PageData {
                gpa: (base + start) as u64,
                compatibility_mask: self.compatibility_mask_all,
                flags,
                data_type: IgvmPageDataType::NORMAL,
                data: data[start..end].to_vec(),
            };
            self.directives.push(page);
        }
        self
    }

    pub fn remove_page(&mut self, base: usize) {
        self.directives.retain(|d| {
            if let IgvmDirectiveHeader::PageData { gpa, .. } = d {
                *gpa as usize != base
            } else {
                true
            }
        })
    }

    pub fn add_data_pages(&mut self, base: usize, data: &[u8]) -> &mut Builder {
        let flags = IgvmPageDataFlags::new();
        self.add_data_pages_flags(base, data, flags)
    }

    pub fn add_data_pages_unmeasured(&mut self, base: usize, data: &[u8]) -> &mut Builder {
        let flags = IgvmPageDataFlags::new().with_unmeasured(true);
        self.add_data_pages_flags(base, data, flags)
    }

    pub fn add_firmware_4g(&mut self, firmware: &[u8]) -> &mut Builder {
        // all below 4GB
        let fwbase = (1u64 << 32) as usize - firmware.len();
        self.add_data_pages(fwbase, firmware);
        self
    }

    pub fn add_firmware_1m(&mut self, firmware: &[u8]) -> &mut Builder {
        // 128k below 1MB
        let lowsize = min(128 * 1024, firmware.len());
        let lowbase = (1u64 << 20) as usize - lowsize;
        let offset = firmware.len() - lowsize;
        self.add_data_pages(lowbase, &firmware[offset..]);
        self
    }

    pub fn add_uefivars(&mut self, uefivars: &[u8], fwsize: usize) -> &mut Builder {
        // varstore (below 4G + firmware code)
        let varsbase = (1u64 << 32) as usize - uefivars.len() - fwsize;
        self.add_data_pages(varsbase, uefivars);
        self
    }

    fn sort_pages(a: &IgvmDirectiveHeader, b: &IgvmDirectiveHeader) -> Ordering {
        let a_gpa = if let IgvmDirectiveHeader::PageData { gpa, .. } = a {
            gpa
        } else {
            &0
        };
        let b_gpa = if let IgvmDirectiveHeader::PageData { gpa, .. } = b {
            gpa
        } else {
            &0
        };
        a_gpa.cmp(b_gpa)
    }

    fn check_pages(&self) -> Result<(), String> {
        let mut pages = HashSet::new();
        for d in &self.directives {
            let page = match d {
                IgvmDirectiveHeader::PageData {
                    gpa,
                    compatibility_mask,
                    ..
                } => Some((*gpa, *compatibility_mask)),
                IgvmDirectiveHeader::ParameterInsert(pi) => Some((pi.gpa, pi.compatibility_mask)),
                _ => None,
            };
            if let Some((gpa, compat)) = page {
                for bit in [NATIVE_COMPAT, SEV_SNP_COMPAT] {
                    if compat & bit != 0 {
                        let entry = (gpa, bit);
                        if pages.contains(&entry) {
                            let err = format!("duplicate at gpa {:x}", gpa);
                            return Err(err);
                        }
                        pages.insert(entry);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn verify(&self) -> Result<(), String> {
        self.check_pages()?;
        Ok(())
    }

    pub fn finalize(&self) -> Result<IgvmFile, igvm::Error> {
        let mut directives = self.directives.clone();
        directives.sort_by(Self::sort_pages);
        IgvmFile::new(
            self.revision,
            self.platforms.clone(),
            self.initializations.clone(),
            directives,
        )
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&IgvmFile> for Builder {
    fn from(igvm: &IgvmFile) -> Builder {
        let mut compatibility_mask_all = 0;
        for p in igvm.platforms() {
            let IgvmPlatformHeader::SupportedPlatform(sp) = p;
            compatibility_mask_all |= sp.compatibility_mask;
        }
        Builder {
            revision: Self::revision(),
            platforms: igvm.platforms().to_vec(),
            initializations: igvm.initializations().to_vec(),
            directives: igvm.directives().to_vec(),
            compatibility_mask_all,
        }
    }
}
