#![cfg(target_os = "uefi")]
//!
//! misc utility functions
//!
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::proto::device_path::DevicePath;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::shell_params::ShellParameters;
use uefi::{print, println};

/// return command line if present
pub fn cmdline() -> Option<Vec<String>> {
    let mut args: Vec<String>;
    if let Ok(params) = boot::open_protocol_exclusive::<ShellParameters>(boot::image_handle()) {
        // started from efi shell
        args = params.args().map(|s| s.to_string()).collect();
        args.remove(0);
    } else {
        let image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle()).ok()?;
        let cmdline16 = image.load_options_as_cstr16().ok()?;
        let cmdline = String::from_utf16_lossy(cmdline16.to_u16_slice());
        args = cmdline.split_whitespace().map(|s| s.to_string()).collect();
    }

    if args.is_empty() {
        None
    } else {
        Some(args)
    }
}

/// connect all controllers
pub fn connect_all(verbose: bool) {
    let Ok(handles) = boot::locate_handle_buffer(boot::SearchType::AllHandles) else {
        return;
    };
    if verbose {
        print!("connecting {} handles ... ", handles.as_ref().len());
    }
    for h in handles.as_ref() {
        let _ = boot::connect_controller(*h, &[], None, true);
    }
    if verbose {
        println!("done");
    }
}

/// print device path for a given handle
pub fn print_handle_devpath(prefix: &str, handle: &Handle) {
    let Ok(dp) = boot::open_protocol_exclusive::<DevicePath>(*handle) else {
        println!("{}no device path for handle", prefix);
        return;
    };
    println!("{}{}", prefix, dp);
}
