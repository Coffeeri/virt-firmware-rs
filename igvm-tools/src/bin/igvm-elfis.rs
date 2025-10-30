//!
//! convert elf images into igvm
//!
use clap::Parser;
use log::{debug, error, info};
use object::elf::{FileHeader64, PT_LOAD, SHF_ALLOC};
use object::read::elf::{ElfFile, ProgramHeader, SectionHeader};
use object::read::File;
use object::{Endianness, Object, ObjectSection};
use std::fs;
use std::process::ExitCode;

use virtfw_igvm_tools::builder::Builder;
use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::x86regs::flat32_mode_regs;

const USE_FLAT32: bool = false;

const IGVM_PARAM_MEMMAP: &str = ".igvm.memmap";

#[derive(Parser, Debug)]
#[command(version, name = "igvm-elfis", about = "convert elf images into igvm", long_about = None)]
struct Args {
    /// list regions in elf image
    #[arg(long)]
    regions: bool,

    /// inspect generated igvm
    #[arg(long)]
    inspect: bool,

    /// input elf image
    #[arg(short, long, value_name = "ELF")]
    input: String,

    /// output igvm image
    #[arg(short, long, value_name = "IGVM")]
    output: Option<String>,
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
}

fn is_rom_addr(addr: &u64) -> bool {
    // 16 MB below 4G
    (0xff00_0000..0xffff_ffff).contains(addr)
}

fn parse_elf(elf: ElfFile<FileHeader64<Endianness>>) -> Option<Vec<Region>> {
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

        let vaddr = ph.p_vaddr(en);
        let paddr = ph.p_paddr(en);
        let memsz = ph.p_memsz(en);
        debug!(
            "{:20} | {:8x} {:8x} {:6x} | {:<}",
            "PT_LOAD", vaddr, paddr, memsz, "-"
        );

        for section in elf.sections() {
            let sh = section.elf_section_header();
            let flags = sh.sh_flags(en);
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

fn main() -> ExitCode {
    let cfg = Args::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();

    info!("reading {}", &cfg.input);
    let binary = fs::read(cfg.input).expect("input: file read error");
    let object = File::parse(&*binary).expect("binary parse error");
    let File::Elf64(elf) = object else {
        error!("not an elf64 file");
        return ExitCode::from(1);
    };

    info!("parsing elf");
    let regions = parse_elf(elf).unwrap();

    if cfg.regions {
        info!("regions found");
        println!("{:<12}  {:>8}  {:>8}  {:>6}", "Name", "VMA", "LMA", "Size");
        println!("-----------------------------------------");
        for r in &regions {
            println!("{:<12}  {}  {}  {:6x}", r.name, r.vma(), r.lma(), r.size);
        }
    }

    info!("building igvm");
    let mut builder = Builder::new();
    builder.add_native_platform();

    if USE_FLAT32 {
        let regs = flat32_mode_regs();
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
        builder.add_empty_normal_pages(r.vma.unwrap() as usize, r.size as usize);
    }

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

    let igvm_res = builder.finalize();
    if let Err(e) = igvm_res {
        error!("build igvm: {e}");
        return ExitCode::from(1);
    }
    let igvm = igvm_res.unwrap();

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
