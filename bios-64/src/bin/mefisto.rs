#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use core::ops::Range;
use core::ptr::addr_of_mut;
use log::info;

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
use virtfw_bios_64::{println, symrange};
use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::FwCfg;
use virtfw_libhw::qemuacpi::QemuAcpi;

const KIB: usize = 1 << 10;
const MIB: usize = 1 << 20;
const GIB: usize = 1 << 30;

const BS_POOL_SIZE: usize = 2 * MIB;
const RT_POOL_SIZE: usize = 2 * MIB;

static CONSOLE: Console = Console::new();

#[link_section = ".pool.rt"]
static mut RT_POOL: [MaybeUninit<u8>; RT_POOL_SIZE] = [MaybeUninit::uninit(); RT_POOL_SIZE];

#[link_section = ".pool.bs"]
static mut BS_POOL: [MaybeUninit<u8>; BS_POOL_SIZE] = [MaybeUninit::uninit(); BS_POOL_SIZE];

fn pretty_size(range: &Range<usize>) -> (usize, &'static str) {
    let size = range.end - range.start;
    if size > 10 * GIB {
        (size / GIB, "G")
    } else if size > 10 * MIB {
        (size / MIB, "M")
    } else if size > 10 * KIB {
        (size / KIB, "k")
    } else {
        (size, "")
    }
}

fn print_range(prefix: &str, range: &Range<usize>) {
    let (value, unit) = pretty_size(range);
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
        let mut base = fwcfg.read64_le() as usize;
        let mut size = fwcfg.read64_le() as usize;
        let kind = fwcfg.read32_le();

        if base < MIB {
            // quirk: ignore real mode address space
            base += MIB;
            size -= MIB;
        }

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
    let pool_rt = symrange!(pool_rt_start, pool_rt_end);
    let pool_bs = symrange!(pool_bs_start, pool_bs_end);

    print_range("firmware/flash", &rom);
    print_range("firmware/code", &code);
    print_range("firmware/data", &data);

    print_range("memory/heap", &heap);
    print_range("memory/stack", &stack);
    print_range("memory/pool/rt", &pool_rt);
    print_range("memory/pool/bs", &pool_bs);

    if false {
        // reserve i386 platform range
        let res = 640 * KIB..MIB;
        print_range("i386/reserved", &res);
        memmap
            .allocate_region(&res, EfiUnusableMemory, 0)
            .expect("add reserved");
    }

    // reserve and remap my own memory
    for (r, t, a) in [
        (&code, EfiRuntimeServicesCode, EFI_MEMORY_RO),
        (&data, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&stack, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&heap, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&pool_rt, EfiRuntimeServicesData, EFI_MEMORY_XP),
        (&pool_bs, EfiBootServicesData, EFI_MEMORY_XP),
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

    info!("efi memory map complete");
}

#[no_mangle]
fn main() {
    sercon_init();
    exception_init();
    alloc_init();

    let fwcfg = FwCfgX86::new();
    let mapper = mapper::MemoryMapper::new();
    let memmap = MemoryMap::new();

    memory_detect(&memmap, &mapper, &fwcfg);
    memory_init(&memmap, &mapper);

    let rng = rng::Random::new();
    let efi = efiloader::init(Some(&CONSOLE), memmap, mapper, rng).expect("init efi runtime");
    info!("efi loader init complete");

    let cmdline_str = "console=ttyS0 panic=1 loglevel=6";
    let cmdline_efi = cmdline_str.encode_utf16().collect::<Vec<u16>>();

    if let Some(loader) = loader::FwCfgLoader::new(&fwcfg) {
        if let Some(mut li) = efi.load_image(&loader) {
            info!("starting image");
            li.set_load_options(cmdline_efi);
            let ret = li.start_image();
            info!("image exited, return code: {ret:?}");
        }
    }

    if let Some(acpi) = QemuAcpi::try_new() {
        acpi.poweroff();
    }
}
