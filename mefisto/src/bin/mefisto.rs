#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;

use core::mem::MaybeUninit;
use core::ops::Range;
use core::ptr::addr_of_mut;
use log::{debug, error, info};

use efiloader::memmap::{MemoryMap, Placement};
use efiloader::memorytype::EfiMemoryType::{self, *};
use efiloader::memorytype::{EFI_MEMORY_RO, EFI_MEMORY_XP};
use efiloader::EfiContext;
use efiloader::MemoryMapper;
use efiloader::{guid, Guid};

use igvm_defs::{MemoryMapEntryType, IGVM_VHS_MEMORY_MAP_ENTRY};

use virtfw_libefi::hob::igvmdata::EfiIgvmDataType;
use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::FwCfg;
use virtfw_libhw::fwcfg::{acpi, smbios};
use virtfw_libhw::pci::setup::PciSetup;
use virtfw_libhw::qemuboards::{qemu_probe, QemuBoard};
use virtfw_libhw::size::*;
use virtfw_libhw::x86cpuid::cpu_phys_bits;
use virtfw_mefisto::alloc::alloc_init;
use virtfw_mefisto::efi::console::Console;
use virtfw_mefisto::efi::loader;
use virtfw_mefisto::efi::mapper;
use virtfw_mefisto::efi::rng;
use virtfw_mefisto::exception::exception_init;
use virtfw_mefisto::sercon::sercon_init;
use virtfw_mefisto::{println, symrange};

const META_BIOS_MAJOR: u8 = 25;
const META_BIOS_MINOR: u8 = 10;
const META_BIOS_DATE: &str = "10/22/2025";
const META_BIOS_NAME: &str = "mefisto/efiloader";
const META_CRATE_VER: &str = env!("CARGO_PKG_VERSION");

const SMBIOS_GUID: Guid = guid!(
    0xeb9d2d31,
    0x2d88,
    0x11d3,
    [0x9a, 0x16, 0x0, 0x90, 0x27, 0x3f, 0xc1, 0x4d]
);

const SMBIOS3_GUID: Guid = guid!(
    0xf2fd1544,
    0x9794,
    0x4a2c,
    [0x99, 0x2e, 0xe5, 0xbb, 0xcf, 0x20, 0xe3, 0x94]
);

const RSDP_GUID: Guid = guid!(
    0x8868e871,
    0xe4f1,
    0x11d3,
    [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81]
);

const BS_POOL_SIZE: usize = MIB;
const RT_POOL_SIZE: usize = MIB;
const MEMORY_MAP_MAX: usize = PAGE_4K / size_of::<IGVM_VHS_MEMORY_MAP_ENTRY>();

static CONSOLE: Console = Console::new();
static FWCFG: FwCfgX86 = FwCfgX86::new();

#[link_section = ".pool.rt"]
static mut RT_POOL: [MaybeUninit<u8>; RT_POOL_SIZE] = [MaybeUninit::uninit(); RT_POOL_SIZE];

#[link_section = ".pool.bs"]
static mut BS_POOL: [MaybeUninit<u8>; BS_POOL_SIZE] = [MaybeUninit::uninit(); BS_POOL_SIZE];

#[link_section = ".igvm.memmap"]
static MEMORY_MAP: MaybeUninit<[IGVM_VHS_MEMORY_MAP_ENTRY; MEMORY_MAP_MAX]> = MaybeUninit::uninit();

fn print_range(prefix: &str, range: &Range<usize>) {
    let (value, unit) = pretty_range(range);
    info!(
        "  {prefix:16} :  0x{:08x} - 0x{:08x}  [{value}{unit}]",
        range.start, range.end,
    );
}

fn memory_detect_igvm() -> Option<Vec<(Range<usize>, EfiMemoryType)>> {
    let mmap = unsafe { MEMORY_MAP.assume_init_ref() };
    let mut memory = Vec::new();

    for m in mmap {
        if m.number_of_pages == 0 || m.reserved != 0 {
            break;
        }
        let base = m.starting_gpa_page_number as usize * PAGE_4K;
        let size = m.number_of_pages as usize * PAGE_4K;
        let range = base..base + size;
        match m.entry_type {
            MemoryMapEntryType::MEMORY => {
                print_range("igvm/memory", &range);
                memory.push((range, EfiConventionalMemory));
            }
            MemoryMapEntryType::PLATFORM_RESERVED => {
                print_range("igvm/reserved", &range);
                memory.push((range, EfiReservedEfiMemoryType));
            }
            _ => {
                print_range("igvm/*UNKNOWN*", &range);
            }
        }
    }

    if memory.is_empty() {
        None
    } else {
        Some(memory)
    }
}

fn memory_detect_fwcfg(fwcfg: &FwCfgX86) -> Vec<(Range<usize>, EfiMemoryType)> {
    // read e820 memory map from fwcfg
    let mut memory = Vec::new();
    let (item, size) = fwcfg.findfile("etc/e820").expect("find e820");
    debug!("fwcfg: etc/e820 is item {item}, size {size}");
    fwcfg.select(item);
    for _ in 0..size / 20 {
        let base = fwcfg.read64_le() as usize;
        let size = fwcfg.read64_le() as usize;
        let kind = fwcfg.read32_le();
        let range = base..base + size;
        match kind {
            1 => {
                print_range("e820/memory", &range);
                memory.push((range, EfiConventionalMemory));
            }
            2 => {
                print_range("e820/reserved", &range);
                memory.push((range, EfiReservedEfiMemoryType));
            }
            _ => {
                print_range("e820/*UNKNOWN*", &range);
            }
        }
    }
    memory
}

fn memory_detect(fwcfg: &FwCfgX86) -> Vec<(Range<usize>, EfiMemoryType)> {
    if let Some(m) = memory_detect_igvm() {
        return m;
    }
    memory_detect_fwcfg(fwcfg)
}

fn memory_phys_init(
    memmap: &MemoryMap,
    mapper: &mapper::MemoryMapper,
    memory: &[(Range<usize>, EfiMemoryType)],
) {
    // process memory map
    for (range, kind) in memory {
        match kind {
            EfiConventionalMemory => {
                memmap.declare_memory_region(range).expect("add memory");
                if range.start >= 4 * GIB {
                    // we boot with the first 4GB already identity
                    // mapped, so only memory above that must be
                    // mapped.
                    mapper.map_range(range);
                }
            }
            EfiReservedEfiMemoryType => {
                memmap
                    .declare_runtime_region(range, EfiReservedEfiMemoryType, 0, 0)
                    .expect("reserve range");
            }
            _ => {}
        }
    }
}

fn memory_below_4g_max(memory: &[(Range<usize>, EfiMemoryType)]) -> usize {
    memory
        .iter()
        .filter(|m| m.0.end < 4 * GIB)
        .filter(|m| m.1 == EfiConventionalMemory)
        .map(|m| m.0.end)
        .max()
        .unwrap()
}

fn memory_above_4g_max(memory: &[(Range<usize>, EfiMemoryType)]) -> usize {
    memory
        .iter()
        .filter(|m| m.0.end > 4 * GIB)
        .filter(|m| m.1 == EfiConventionalMemory)
        .map(|m| m.0.end)
        .max()
        .unwrap_or(4 * GIB)
}

fn memory_virt_init(memmap: &MemoryMap, mapper: &mapper::MemoryMapper) {
    let rom = symrange!(copy_from, flash_end);
    let code = symrange!(copy_start, data_start);
    let data = symrange!(data_start, copy_end);
    let stack = symrange!(stack_start, stack_end);
    let heap = symrange!(heap_start, heap_end);

    let addr_rt = core::ptr::addr_of!(RT_POOL) as usize;
    let addr_bs = core::ptr::addr_of!(BS_POOL) as usize;
    let pool_rt = addr_rt..addr_rt + RT_POOL_SIZE;
    let pool_bs = addr_bs..addr_bs + BS_POOL_SIZE;

    // i386 platform range
    let reserved = 640 * KIB..MIB;

    print_range("firmware/flash", &rom);
    print_range("firmware/code", &code);
    print_range("firmware/data", &data);

    print_range("memory/heap", &heap);
    print_range("memory/stack", &stack);
    print_range("memory/pool/rt", &pool_rt);
    print_range("memory/pool/bs", &pool_bs);

    print_range("i386/reserved", &reserved);

    // reserve and remap my own memory
    for (r, t, a) in [
        (&code, EfiRuntimeServicesCode, EFI_MEMORY_RO),
        (&data, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&stack, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&heap, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&pool_rt, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&pool_bs, EfiBootServicesData, EFI_MEMORY_XP),
        (&reserved, EfiReservedEfiMemoryType, 0),
    ] {
        memmap.allocate_region(r, t, a).expect("alloc region");
        mapper.remap_range(r, a, 0).expect("remap region");
    }

    // setup memory pools
    unsafe {
        memmap
            .declare_pool(EfiRuntimeServicesData, &mut *addr_of_mut!(RT_POOL))
            .expect("add rt pool");
        memmap
            .declare_pool(EfiBootServicesData, &mut *addr_of_mut!(BS_POOL))
            .expect("add bs pool");
    };

    // stack guard page (make overflows pagefault)
    mapper.unmap_page(stack.start);
}

fn pci_init(board: &QemuBoard, lowmem: usize, highmem: usize) {
    let mut pcisetup = PciSetup::new(board.pcihost());

    let mut phys_bits = cpu_phys_bits();
    if phys_bits > 47 {
        // with 4-level paging we can not identity-map more than that
        phys_bits = 47;
    }

    let phys_max = 1usize << phys_bits;
    print_range("cpu/physaddrs", &(0..phys_max));

    // 32-bit PCI MMIO window
    let pci32 = if let Some(ecam) = board.ecam() {
        // q35 - max lowmem addr is 0xb000_0000
        lowmem.next_multiple_of(GIB)..ecam.start
    } else {
        // pc - max lowmem addr is 0xe000_0000
        lowmem.next_multiple_of(256 * MIB)..0xfc00_0000
    };
    print_range("pci/mmio32", &pci32);
    pcisetup.configure_mem32(pci32);

    // 64-bit PCI MMIO window (1/8 of physical address space)
    let size64 = phys_max / 8;
    let start64 = highmem.next_power_of_two().next_multiple_of(size64);
    if start64 + size64 < phys_max {
        let pci64 = start64..start64 + size64;
        print_range("pci/mmio64", &pci64);
        pcisetup.configure_mem64(pci64);
    }

    info!("setting up pci bus");
    pcisetup.enumerate_all();
    pcisetup.resources_all();
    pcisetup.assign_all();
    pcisetup.apply_all();
}

fn qemu_init(memmap: &MemoryMap, lowmem: usize, highmem: usize) -> Option<QemuBoard> {
    let board = qemu_probe()?;
    info!("running on: {}", board.name());
    board.init_acpi();
    if let Some(ecam) = board.ecam() {
        print_range("q35/mmconfig", &ecam);
        memmap
            .declare_runtime_region(&ecam, EfiReservedEfiMemoryType, 0, 0)
            .expect("reserve mmconfig");
    }
    pci_init(&board, lowmem, highmem);
    Some(board)
}

fn efi_acpi_alloc(efi: &EfiContext, size: usize) -> Option<&'static mut [u8]> {
    let limit = Placement::Max(4 * GIB as u64);
    let pages = size.div_ceil(4096);
    let uninit = efi.allocate_pages(pages, EfiACPIReclaimMemory, limit)?;
    let slice = unsafe { core::slice::from_raw_parts_mut(uninit.as_ptr() as *mut _, uninit.len()) };
    Some(slice)
}

#[no_mangle]
fn main() {
    sercon_init(log::LevelFilter::Info);
    println!("");
    println!("This is {META_BIOS_NAME}, v{META_CRATE_VER}");
    println!("");

    exception_init();
    alloc_init();

    if !FWCFG.detect() {
        panic!("fwcfg: not present");
    }

    let mapper = mapper::MemoryMapper::new();
    let memmap = MemoryMap::new();

    let memory = memory_detect(&FWCFG);
    memory_phys_init(&memmap, &mapper, &memory);
    memory_virt_init(&memmap, &mapper);
    let lowmem = memory_below_4g_max(&memory);
    let highmem = memory_above_4g_max(&memory);
    let board = qemu_init(&memmap, lowmem, highmem);
    info!("efi memory map complete");

    let rng = rng::Random::new();
    let efi = efiloader::init(Some(&CONSOLE), memmap, mapper, rng).expect("init efi runtime");
    info!("efi loader init complete");

    if let Some(mut smbios) = smbios::SmbiosLoader::new(&FWCFG) {
        smbios.set_bios(
            META_BIOS_MAJOR,
            META_BIOS_MINOR,
            META_BIOS_NAME,
            META_CRATE_VER,
            META_BIOS_DATE,
        );
        smbios
            .load_tables(|s, _a| efi_acpi_alloc(efi, s))
            .expect("load tables");
        let (major, minor) = smbios.version().expect("get version");
        let anchor = smbios.anchor().expect("get anchor");
        info!("smbios: version {major}.{minor} anchor at {anchor:?}");
        match major {
            2 => efi.install_configtable(&SMBIOS_GUID, anchor),
            3 => efi.install_configtable(&SMBIOS3_GUID, anchor),
            _ => {}
        }
    }

    if let Some(mut acpi) = acpi::AcpiLoader::new(&FWCFG) {
        acpi.load_tables(|s, _a| efi_acpi_alloc(efi, s))
            .expect("load tables");
        acpi.run_commands().expect("relocate tables");
        let rsdp = acpi.rsdp().expect("get rsdp");
        info!("acpi: rsdp at {rsdp:?}");
        efi.install_configtable(&RSDP_GUID, rsdp);
    }

    // kernel via igvm
    if let Some(igvmkernel) = loader::IgvmFileLoader::new(EfiIgvmDataType::Kernel) {
        info!("igvm: loading kernel image");
        if let Some(mut li) = efi.load_image(&igvmkernel) {
            info!("igvm: starting kernel image");
            let ret = li.start_image();
            error!("igvm: image exited, return code: {ret:?}");
        } else {
            error!("igvm: image load failed");
        }
    }

    // kernel via fwcfg (qemu direct kernel boot)
    let loader = loader::FwCfgLoader::new(&FWCFG);
    let kload = loader.kernel_loader();
    let iload = loader.initrd_loader();
    let cmdline = loader.cmdline();

    if let Some(k) = kload {
        if let Some(i) = iload {
            info!("fwcfg: setup initrd loader");
            efi.set_initrd_loader(i);
        }
        info!("fwcfg: loading kernel image");
        if let Some(mut li) = efi.load_image(&k) {
            li.set_load_options(cmdline);
            info!("fwcfg: starting kernel image");
            let ret = li.start_image();
            error!("fwcfg: image exited, return code: {ret:?}");
        } else {
            error!("fwcfg: image load failed");
        }
    }

    error!("no kernel found (igvm/fwcfg)");
    if let Some(b) = board {
        b.poweroff();
    }
}
