#![cfg(target_os = "uefi")]

extern crate alloc;
use alloc::vec::Vec;

use log::debug;
use uefi::guid;
use uefi::prelude::*;
use uefi::proto::device_path::build;
use uefi::proto::device_path::DevicePath;
use uefi::proto::loaded_image::LoadedImage;

/// detect whenever the uefi application has been loaded via 'qemu -kernel binary.efi'
pub fn is_direct_kernel_load() -> bool {
    // QemuKernelLoaderFsMediaGuid
    let mut vec = Vec::new();
    let qemu_path = build::DevicePathBuilder::with_vec(&mut vec)
        .push(&build::media::Vendor {
            vendor_guid: guid!("1428f772-b64a-441e-b8c3-9ebdd7f893c7"),
            vendor_defined_data: &[],
        })
        .unwrap()
        .finalize()
        .unwrap();
    let qemu_node = qemu_path.node_iter().next().unwrap();

    // load image device path
    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle()).unwrap();
    let img_path =
        boot::open_protocol_exclusive::<DevicePath>(loaded_image.device().unwrap()).unwrap();
    let img_node = img_path.node_iter().next().unwrap();

    // compare and see ;)
    qemu_node == img_node
}

pub fn shutdown_if_direct_kernel_load(wait: bool) {
    if is_direct_kernel_load() {
        if wait {
            debug!("qemu direct kernel boot detected -> wait & poweroff");
            boot::stall(3_000_000);
        } else {
            debug!("qemu direct kernel boot detected -> poweroff");
        }
        runtime::reset(runtime::ResetType::SHUTDOWN, Status::SUCCESS, None);
    };
}
