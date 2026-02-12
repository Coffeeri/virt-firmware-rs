#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use x86_64::structures::mem_encrypt::{enable_memory_encryption, MemoryEncryptionConfiguration};

use virtfw_libhw::qemuboards::qemu_try_poweroff;
use virtfw_libhw::size::PAGE_4K;
use virtfw_libhw::x86cpuid::cpuid;
use virtfw_mefisto::alloc::alloc_init;
use virtfw_mefisto::efi::mapper::MemoryMapper;
use virtfw_mefisto::exception::exception_init;
use virtfw_mefisto::println;
use virtfw_mefisto::sercon::sercon_init;
use virtfw_mefisto::snp::cpuid::{CpuidSevEax, CpuidSevEbx, CPUID};
use virtfw_mefisto::snp::ghcb::{msr_ghcb_page_state, msr_ghcb_sev_info, msr_ghcb_vmm_features};
use virtfw_mefisto::snp::vc::GHCB;

#[link_section = ".snp.secrets"]
pub static mut SECRETS: [MaybeUninit<u8>; PAGE_4K] = [MaybeUninit::uninit(); PAGE_4K];

fn snp_init(mapper: &MemoryMapper) {
    // setup paging
    let info = msr_ghcb_sev_info();
    let cfg = MemoryEncryptionConfiguration::EncryptedBit(info.cbitpos());
    unsafe {
        enable_memory_encryption(cfg);
    }

    // setup ghcb
    let ghcb = GHCB.lock();
    let gpa = ghcb.gpa();
    mapper.share_page(gpa as usize);
    msr_ghcb_page_state(gpa, true);
    ghcb.register();
}

fn snp_info_ghcb() {
    println!("sev info     : {:?}", msr_ghcb_sev_info());
    println!("vmm features : {:?}", msr_ghcb_vmm_features());
    println!("");
}

fn snp_info_cpuid() {
    let (eax, ebx, _ecx, _edx) = cpuid(0x8000_001F, 0);
    let a: CpuidSevEax = eax.into();
    let b: CpuidSevEbx = ebx.into();
    println!("cpuid / eax  : {:?}", a);
    println!("cpuid / ebx  : {:?}", b);
    println!("");
}

fn snp_dump_cpuid() {
    for i in 0..CPUID.count {
        println!("cpuid #{i:02} - {}", CPUID.entries[i as usize]);
    }
    println!("");
}

#[no_mangle]
fn main(boot_mode: u64) {
    exception_init();
    alloc_init();
    let mapper = MemoryMapper::new();

    if boot_mode == 5 {
        snp_init(&mapper);
    }

    sercon_init(log::LevelFilter::Info);
    println!("");
    println!("Hello world!");
    println!("");

    println!("boot mode    : {boot_mode}");
    println!("ghcb page    : {:x}", GHCB.lock().gpa());

    let addr = core::ptr::addr_of!(CPUID) as usize;
    println!("cpuid page   : {addr:x}");

    let addr = core::ptr::addr_of!(SECRETS) as usize;
    println!("secrets page : {addr:x}");
    println!("");

    if boot_mode == 5 {
        snp_info_ghcb();
        snp_info_cpuid();

        if false {
            snp_dump_cpuid();
        }

        // q35 ecam will be mapped here
        mapper.share_range(&(0xe000_0000..0xf000_0000))
    }

    qemu_try_poweroff();
    panic!("Oops, poweroff failed.");
}
