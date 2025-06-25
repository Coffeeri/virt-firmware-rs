#![cfg(target_os = "uefi")]
//!
//! misc utility functions
//!
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::device_path::DevicePath;
use uefi::proto::loaded_image::LoadedImage;
use uefi::{print, println};

/// return command line if present
pub fn cmdline(me: &str) -> Option<String> {
    let image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle()).ok()?;
    let cmdline16 = image.load_options_as_cstr16().ok()?;
    let cmdline = String::from_utf16_lossy(cmdline16.to_u16_slice());
    let mut args: Vec<&str> = cmdline.split_whitespace().collect();

    if let Some(arg0) = args.first() {
        if me == *arg0 {
            args.remove(0);
        }
    }

    Some(args.join(" "))
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
        let _ = boot::connect_controller(*h, None, None, true);
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
    if let Ok(string) = dp.to_string(DisplayOnly(true), AllowShortcuts(true)) {
        println!("{}{}", prefix, string);
    }
}
