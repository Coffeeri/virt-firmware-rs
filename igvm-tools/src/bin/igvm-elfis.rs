//!
//! convert elf images into igvm
//!
use clap::{CommandFactory, Parser};
use igvm_defs::IgvmPageDataType;
use log::{debug, error, info};
use object::elf::{PT_LOAD, SHF_ALLOC};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader, SectionHeader};
use object::read::{File, ObjectSymbol};
use object::{Endianness, Object, ObjectSection};
use std::fs;
use std::process::ExitCode;

use igvm::IgvmFile;
use virtfw_igvm_tools::builder::{Builder, SEV_SNP_COMPAT};
use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::x86regs::flat32_mode_regs;
use virtfw_libefi::hob::igvmdata::{EfiIgvmDataList, EfiIgvmDataType};

const USE_FLAT32: bool = false;

const SNP_CPUID: &str = ".snp.cpuid";
const SNP_SECRETS: &str = ".snp.secrets";
const IGVM_PARAM_MEMMAP: &str = ".igvm.memmap";
const IGVM_DATA_HOB: &str = ".igvm.datahobs";

#[derive(Parser, Debug)]
#[command(version, author, name = "igvm-elfis",
          about = "convert elf images into igvm",
          long_about = None)]
struct Args {
    /// list regions in elf image
    #[arg(long, help_heading = "Print informations")]
    regions: bool,

    /// inspect generated igvm
    #[arg(long, help_heading = "Print informations")]
    inspect: bool,

    /// input elf image
    #[arg(short, long, value_name = "ELF", required_unless_present = "manpage")]
    input: Option<String>,

    /// input efi binary (usually linux kernel)
    #[arg(short, long, value_name = "EFI")]
    kernel: Option<String>,

    /// output igvm image
    #[arg(short, long, value_name = "IGVM")]
    output: Option<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

struct Region<'r> {
    // section
    name: &'r str,
    _addr: u64,
    size: u64,
    data: &'r [u8],
    // segment
    _vaddr: u64,
    _paddr: u64,
    // calculated
    vma: Option<u64>,
    lma: Option<u64>,
}

impl Region<'_> {
    fn fmtaddr(addr: Option<u64>) -> String {
        match addr {
            Some(a) => {
                format!("{a:8x}")
            }
            None => {
                format!("{:>8}", "-")
            }
        }
    }

    fn vma(&self) -> String {
        Self::fmtaddr(self.vma)
    }

    fn lma(&self) -> String {
        Self::fmtaddr(self.lma)
    }

    fn is_igvm_param(&self) -> bool {
        self.name == IGVM_PARAM_MEMMAP
    }

    fn is_igvm_data_hobs(&self) -> bool {
        self.name == IGVM_DATA_HOB
    }
}

fn is_rom_addr(addr: &u64) -> bool {
    // 16 MB below 4G
    (0xff00_0000..0xffff_ffff).contains(addr)
}

fn parse_elf<'e, FH: FileHeader<Endian = Endianness>>(
    elf: &'e ElfFile<'e, FH>,
) -> Option<Vec<Region<'e>>> {
    let mut regions = Vec::new();

    debug!(
        "{:20} | {:>8} {:>8} {:>6} | {:<}",
        "segment/section", "vaddr", "paddr", "size", "kind"
    );
    debug!("------------------------------------------------------");

    for segment in elf.segments() {
        let en = elf.endianness();
        let ph = segment.elf_program_header();
        if ph.p_type(en) != PT_LOAD {
            error!("unknown elf header type: {}", ph.p_type(en));
            return None;
        }

        let vaddr: u64 = ph.p_vaddr(en).into();
        let paddr: u64 = ph.p_paddr(en).into();
        let memsz: u64 = ph.p_memsz(en).into();
        debug!(
            "{:20} | {:8x} {:8x} {:6x} | {:<}",
            "PT_LOAD", vaddr, paddr, memsz, "-"
        );

        for section in elf.sections() {
            let sh = section.elf_section_header();
            let flags: u64 = sh.sh_flags(en).into();
            if flags & SHF_ALLOC as u64 == 0 {
                // ignore debug info, symbol table etc.
                continue;
            }

            // segment filter
            let addr = section.address();
            let size = section.size();
            if addr < vaddr {
                continue;
            }
            if addr + size > vaddr + memsz {
                continue;
            }

            let name = section.name().unwrap_or("?");
            debug!(
                "    {:16} | {:8x} {:>8} {:6x} | {:?}",
                name,
                addr,
                "-",
                size,
                section.kind(),
            );

            // calculate vma + lma
            let have_data = !section.data().unwrap().is_empty();
            let vma = if vaddr != paddr || !have_data {
                Some(addr)
            } else {
                None
            };
            let lma = if have_data {
                Some(addr - vaddr + paddr)
            } else {
                None
            };

            let region = Region {
                name,
                _addr: addr,
                size,
                data: section.data().unwrap(),
                _vaddr: vaddr,
                _paddr: paddr,
                vma,
                lma,
            };
            regions.push(region);
        }
    }
    Some(regions)
}

fn igvm_add_kernel(builder: &mut Builder, hobs_region: &Region, kernel_blob: Option<&[u8]>) {
    let mut hoblist = EfiIgvmDataList::new(0x20000000); // start at 512 MB

    if let Some(b) = kernel_blob {
        info!("add efi data: kernel");
        hoblist.add(b, EfiIgvmDataType::Kernel, false);
    }

    if hoblist.is_empty() {
        return;
    }

    let addr = hobs_region.vma.unwrap() as usize;
    info!("add efi data: hoblist at 0x{addr:x}");
    let hobs_blob = hoblist.hobs();
    builder.add_data_pages(addr, &hobs_blob);

    // add data pages
    for (addr, blob) in hoblist.blobs(true) {
        builder.add_data_pages(addr, blob);
    }
    for (addr, blob) in hoblist.blobs(false) {
        builder.add_data_pages_unmeasured(addr, blob);
    }
}

fn entry_addr(object: &object::File, symbol: &str) -> Option<u64> {
    let addr = object
        .symbols()
        .find(|s| s.name().unwrap_or("") == symbol)
        .map(|s| s.address());
    if let Some(a) = addr {
        info!("entry: symbol {} -> address 0x{:x}", symbol, a);
    }
    addr
}

fn process(cfg: &Args) -> Result<IgvmFile, Box<dyn std::error::Error>> {
    let input = cfg.input.as_ref().unwrap();
    info!("reading {}", input);
    let binary = fs::read(input)?;
    let object = File::parse(&*binary)?;

    info!("parsing elf");
    let regions = match &object {
        File::Elf64(elf) => parse_elf(elf).unwrap(),
        File::Elf32(elf) => parse_elf(elf).unwrap(),
        _ => {
            return Err("not an elf file".into());
        }
    };

    if cfg.regions {
        info!("regions found");
        println!("{:<16}  {:>8}  {:>8}  {:>6}", "Name", "VMA", "LMA", "Size");
        println!("--------------------------------------------");
        for r in &regions {
            println!("{:<16}  {}  {}  {:6x}", r.name, r.vma(), r.lma(), r.size);
        }
    }

    info!("building igvm");
    let mut builder = Builder::new();
    builder.add_native_platform();

    let snp = regions.iter().any(|r| r.name == SNP_SECRETS);
    if snp {
        let entry = entry_addr(&object, "entry_snp");
        let regs = flat32_mode_regs(entry);
        builder.add_snp_platform();
        builder.add_snp_vmsa_context(&regs, false);
    }

    if USE_FLAT32 {
        let regs = flat32_mode_regs(None);
        builder.add_native_context(&regs);
    }

    // firmware rom pages
    if let Some(base) = regions
        .iter()
        .filter_map(|r| r.lma)
        .filter(is_rom_addr)
        .min()
    {
        let size = 0x1_0000_0000 - base as usize;
        info!("flash/rom at {base:x} +{size:06x}");
        let mut rom = vec![0; size];
        for r in regions
            .iter()
            .filter(|r| r.lma.is_some())
            .filter(|r| is_rom_addr(&r.lma.unwrap()))
        {
            let offset = (r.lma.unwrap() - base) as usize;
            rom[offset..offset + r.data.len()].copy_from_slice(r.data)
        }
        if !USE_FLAT32 {
            builder.add_firmware_1m(&rom);
        }
        builder.add_firmware_4g(&rom);
    }

    //
    // ram regions used
    //
    // Doesn't do much for native mode.  For confidential vms that
    // will be pre-accepted pages, also special pages such as cpuid,
    // secrets and igvm parameters will be sections in the elf binary.
    //
    for r in regions
        .iter()
        .filter(|r| r.vma.is_some())
        .filter(|r| !r.is_igvm_param())
    {
        match r.name {
            SNP_CPUID => {
                builder.add_empty_pages(
                    r.vma.unwrap() as usize,
                    r.size as usize,
                    SEV_SNP_COMPAT,
                    IgvmPageDataType::CPUID_DATA,
                );
            }
            SNP_SECRETS => {
                builder.add_empty_pages(
                    r.vma.unwrap() as usize,
                    r.size as usize,
                    SEV_SNP_COMPAT,
                    IgvmPageDataType::SECRETS,
                );
            }
            _ => {
                builder.add_empty_normal_pages(r.vma.unwrap() as usize, r.size as usize);
            }
        }
    }

    // IGVM parameters
    let params: Vec<_> = regions.iter().filter(|r| r.is_igvm_param()).collect();
    if !params.is_empty() {
        let s = params
            .iter()
            .map(|r| r.vma.unwrap())
            .min()
            .expect("params start");
        let e = params
            .iter()
            .map(|r| r.vma.unwrap() + r.size)
            .max()
            .expect("params end")
            .next_multiple_of(4096);
        info!("igvm params: 0x{s:x} -> 0x{e:x}");

        builder.add_igvm_param_area(0, e - s);
        for p in params {
            if p.name == IGVM_PARAM_MEMMAP {
                builder.add_igvm_param_memmap(0, (p.vma.unwrap() - s) as u32);
            }
        }
        builder.add_igvm_param_insert(0, s);
    };

    // additional data: kernel
    let kernel_blob = if let Some(kernel_name) = &cfg.kernel {
        info!("reading {}", &kernel_name);
        let data = std::fs::read(kernel_name).expect("kernel: file read error");
        Some(data)
    } else {
        None
    };

    // additional data: HOBs
    if let Some(region) = regions.iter().find(|r| r.is_igvm_data_hobs()) {
        igvm_add_kernel(&mut builder, region, kernel_blob.as_deref());
    }

    // constructing igvm is complete
    let igvm = builder.finalize()?;
    Ok(igvm)
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
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();

    let igvm = match process(&cfg) {
        Err(e) => {
            error!("{e}");
            return 1.into();
        }
        Ok(i) => i,
    };

    if cfg.inspect {
        info!("inspect igvm");
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
    }

    ExitCode::from(0)
}
