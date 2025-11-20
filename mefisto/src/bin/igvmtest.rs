#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]
use core::mem::size_of;
use core::mem::MaybeUninit;
use zerocopy::FromBytes;

use igvm_defs::{IGVM_VHS_MEMORY_MAP_ENTRY, PAGE_SIZE_4K};

use virtfw_mefisto::println;
use virtfw_mefisto::sercon::sercon_init;
use virtfw_libhw::qemuboards::qemu_try_poweroff;

const MEMORY_MAP_MAX: usize = PAGE_SIZE_4K as usize / size_of::<IGVM_VHS_MEMORY_MAP_ENTRY>();

#[link_section = ".igvm.memmap"]
static MEMORY_MAP: MaybeUninit<[IGVM_VHS_MEMORY_MAP_ENTRY; MEMORY_MAP_MAX]> = MaybeUninit::uninit();

#[link_section = ".igvm.datahobs"]
static DATA_HOBS: MaybeUninit<[u8; PAGE_SIZE_4K as usize]> = MaybeUninit::uninit();

// FIXME: copy from igvm-tools/src/efidata.rs
#[derive(Debug, FromBytes)]
#[repr(C)]
pub struct EfiIgvmDataHob {
    hob_type: u16,
    hob_length: u16,
    _reserved: u32,
    guid_ext: [u8; 16],
    address: u64,
    length: u64,
    data_type: u32,
    data_flags: u32,
}

fn print_memory_map() {
    let mmap = unsafe { MEMORY_MAP.assume_init_ref() };

    println!("igvm memory map parameter entries");
    for m in mmap {
        if m.number_of_pages == 0 || m.reserved != 0 {
            break;
        }
        println!(
            "  first page 0x{:x}, page count 0x{:x}, type {:?}, flags {}",
            m.starting_gpa_page_number, m.number_of_pages, m.entry_type, m.flags
        );
    }
    println!("");
}

fn print_data_hobs() {
    let hobs = unsafe { DATA_HOBS.assume_init_ref() };
    let mut left: &[u8] = hobs;

    println!("igvm efi data hobs");
    while left.len() > size_of::<EfiIgvmDataHob>() {
        let (hob, r) = EfiIgvmDataHob::read_from_prefix(left).unwrap();
        if (hob.hob_type != 0x0004) || (hob.hob_length != size_of::<EfiIgvmDataHob>() as u16) {
            break;
        }
        println!(
            "  address 0x{:x}, length 0x{:x}, type 0x{:x}",
            hob.address, hob.length, hob.data_type
        );
        left = r;
    }
    println!("");
}

#[no_mangle]
fn main() {
    sercon_init(log::LevelFilter::Info);

    println!("");
    println!("IGVM test firmware image");
    println!("");

    print_memory_map();
    print_data_hobs();

    qemu_try_poweroff();
}
