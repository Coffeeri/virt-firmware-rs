//!
//! vmfwupdate test case
//!
#![allow(dead_code)]
#![no_main]
#![no_std]

extern crate alloc;
#[cfg(target_arch = "x86_64")]
use alloc::string::ToString;
#[cfg(target_arch = "x86_64")]
use alloc::vec;
use alloc::vec::Vec;

use log::{debug, error};
use uefi::fs::FileSystem;
use uefi::prelude::*;
use uefi::println;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::CString16;

#[cfg(target_arch = "x86_64")]
use virtfw_efi_apps::misc;
use virtfw_efi_apps::qemu;
use virtfw_libhw::fwcfg::vmfwupdate::FirmwareLoader;
use virtfw_libhw::fwcfg::FwCfg;

#[cfg(target_arch = "x86_64")]
use virtfw_libhw::fwcfg::x86::FwCfgX86;

fn vmfwupdate(fwcfg: &dyn FwCfg, igvm: &[u8]) {
    let Some(mut loader) = FirmwareLoader::new(fwcfg) else {
        println!("vmfwupdate: not supported");
        return;
    };

    println!("vmfwupdate: is supported");
    println!("  version : {}", loader.get_version());
    println!("  caps    : {:?}", loader.get_caps());

    println!("vmfwupdate: setup igvm");
    loader.set_igvm(igvm, true);

    println!("vmfwupdate: launch igvm (via reset)");
    runtime::reset(runtime::ResetType::COLD, Status::SUCCESS, None);
}

fn loadfile(filename: &str) -> Option<Vec<u8>> {
    let handle = boot::image_handle();
    let li = boot::open_protocol_exclusive::<LoadedImage>(handle).ok()?;
    let sfs = boot::open_protocol_exclusive::<SimpleFileSystem>(li.device()?).ok()?;
    let mut fs = FileSystem::new(sfs);
    debug!("filesystem: ok");

    let cstr16 = CString16::try_from(filename).unwrap();
    let blob = match fs.read(&*cstr16) {
        Ok(d) => {
            debug!("read '{}': ok", filename);
            Some(d)
        }
        Err(e) => {
            error!("read '{}': {}", filename, e);
            None
        }
    };

    blob
}

const HELLO_POWEROFF: &[u8] = include_bytes!("../../../bios-32/hello-poweroff.igvm");
const HELLO_RESET: &[u8] = include_bytes!("../../../bios-32/hello-reset.igvm");

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    println!();
    println!("=== vmfwupdate test utility ===");
    println!();

    #[cfg(target_arch = "x86_64")]
    {
        let fwcfg = FwCfgX86::new();

        let default = vec!["poweroff".to_string()];
        let cmdline = misc::cmdline().unwrap_or(default);
        let arg1 = cmdline.first().unwrap();
        match arg1.as_str() {
            "poweroff" => {
                println!("using builtin igvm [poweroff] ...");
                vmfwupdate(&fwcfg, HELLO_POWEROFF);
            }
            "reset" => {
                println!("using builtin igvm [reset] ...");
                vmfwupdate(&fwcfg, HELLO_RESET);
            }
            _ => {
                println!("loading igvm {} ...", arg1);
                if let Some(igvm) = loadfile(arg1) {
                    vmfwupdate(&fwcfg, &igvm);
                }
            }
        }
    }

    qemu::shutdown_if_direct_kernel_load(true);
    Status::SUCCESS
}
