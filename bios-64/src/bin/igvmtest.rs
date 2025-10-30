#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]
use core::mem::size_of;
use core::mem::MaybeUninit;

use igvm_defs::{IGVM_VHS_MEMORY_MAP_ENTRY, PAGE_SIZE_4K};

use virtfw_bios_64::println;
use virtfw_bios_64::sercon::sercon_init;
use virtfw_libhw::qemuboards::qemu_try_poweroff;

const MEMORY_MAP_MAX: usize = PAGE_SIZE_4K as usize / size_of::<IGVM_VHS_MEMORY_MAP_ENTRY>();

#[link_section = ".igvm.memmap"]
static MEMORY_MAP: MaybeUninit<[IGVM_VHS_MEMORY_MAP_ENTRY; MEMORY_MAP_MAX]> = MaybeUninit::uninit();

fn print_memory_map() {
    let mmap = unsafe { MEMORY_MAP.assume_init_ref() };

    println!("igvm memory map parameter entries");
    for m in mmap {
        if m.number_of_pages == 0 || m.reserved != 0 {
            break;
        }
        println!(
            "  first page {:x}, page count {:x}, type {:?}, flags {}",
            m.starting_gpa_page_number, m.number_of_pages, m.entry_type, m.flags
        );
    }
}

#[no_mangle]
fn main() {
    sercon_init(log::LevelFilter::Info);

    println!("");
    println!("IGVM test firmware image");
    println!("");

    print_memory_map();

    qemu_try_poweroff();
}
