#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::ops::Range;
use core::ptr::addr_of_mut;
use log::{error, info};

use efiloader::memmap::MemoryMap;
use efiloader::memorytype::EfiMemoryType::*;
use efiloader::memorytype::{EFI_MEMORY_RO, EFI_MEMORY_XP};
use efiloader::MemoryMapper;

use virtfw_bios_64::alloc::alloc_init;
use virtfw_bios_64::efi::console::Console;
use virtfw_bios_64::efi::loader;
use virtfw_bios_64::efi::mapper;
use virtfw_bios_64::efi::rng;
use virtfw_bios_64::exception::exception_init;
use virtfw_bios_64::sercon::sercon_init;
use virtfw_bios_64::size::*;
use virtfw_bios_64::{println, symrange};
use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::FwCfg;
use virtfw_libhw::qemuboards::{qemu_probe, QemuBoard};

const BS_POOL_SIZE: usize = MIB;
const RT_POOL_SIZE: usize = MIB;

static CONSOLE: Console = Console::new();
static FWCFG: FwCfgX86 = FwCfgX86::new();

#[link_section = ".pool.rt"]
static mut RT_POOL: [MaybeUninit<u8>; RT_POOL_SIZE] = [MaybeUninit::uninit(); RT_POOL_SIZE];

#[link_section = ".pool.bs"]
static mut BS_POOL: [MaybeUninit<u8>; BS_POOL_SIZE] = [MaybeUninit::uninit(); BS_POOL_SIZE];

fn print_range(prefix: &str, range: &Range<usize>) {
    let (value, unit) = pretty_range(range);
    println!(
        "  {prefix:16} :  0x{:08x} - 0x{:08x}  [{value}{unit}]",
        range.start, range.end,
    );
}

fn memory_detect(memmap: &MemoryMap, mapper: &mapper::MemoryMapper, fwcfg: &FwCfgX86) {
    // read e820 memory map
    let (item, size) = fwcfg.findfile("etc/e820").expect("find e820");
    info!("fwcfg: etc/e820 is item {item}, size {size}");
    fwcfg.select(item);
    for _ in 0..size / 20 {
        let base = fwcfg.read64_le() as usize;
        let size = fwcfg.read64_le() as usize;
        let kind = fwcfg.read32_le();

        let r = base..base + size;
        match kind {
            1 => {
                print_range("e820/memory", &r);
                memmap.declare_memory_region(&r).expect("add memory");
                if r.start >= 4 * GIB {
                    mapper.map_range(&r);
                }
            }
            2 => {
                print_range("e820/reserved", &r);
            }
            _ => {
                print_range("e820/*UNKNOWN*", &r);
            }
        }
    }
}

fn memory_init(memmap: &MemoryMap, mapper: &mapper::MemoryMapper) {
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

    info!("efi memory map complete");
}

fn qemu_init(memmap: &MemoryMap) -> Option<QemuBoard> {
    let board = qemu_probe()?;
    info!("running on: {}", board.name());
    board.init_acpi();
    if let Some(ecam) = board.ecam() {
        print_range("q35/mmconfig", &ecam);
        memmap
            .declare_runtime_region(&ecam, EfiReservedEfiMemoryType, 0, 0)
            .expect("reserve mmconfig");
    }
    Some(board)
}

#[no_mangle]
fn main() {
    sercon_init();
    exception_init();
    alloc_init();

    let mapper = mapper::MemoryMapper::new();
    let memmap = MemoryMap::new();

    memory_detect(&memmap, &mapper, &FWCFG);
    memory_init(&memmap, &mapper);
    let board = qemu_init(&memmap);

    let rng = rng::Random::new();
    let efi = efiloader::init(Some(&CONSOLE), memmap, mapper, rng).expect("init efi runtime");
    info!("efi loader init complete");

    let loader = loader::FwCfgLoader::new(&FWCFG);
    let kload = loader.kernel_loader();
    let iload = loader.initrd_loader();
    let cmdline = loader.cmdline();

    if let Some(i) = iload {
        info!("setup initrd loader");
        efi.set_initrd_loader(i);
    }
    if let Some(k) = kload {
        info!("loading kernel image");
        if let Some(mut li) = efi.load_image(&k) {
            li.set_load_options(cmdline);
            info!("starting kernel image");
            let ret = li.start_image();
            info!("image exited, return code: {ret:?}");
        }
    } else {
        error!("no kernel found in fwcfg");
    }

    if let Some(b) = board {
        b.poweroff();
    }
}
