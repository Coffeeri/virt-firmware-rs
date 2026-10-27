extern crate alloc;
use alloc::string::String;

use efiloader::status::Status;
use efiloader::{Char16, Guid};
use uefi::CString16;

use virtfw_libefi::efivar::types::{EfiVar, EfiVarId};

fn ptr_to_string(ptr: *const Char16) -> String {
    let uefiptr = ptr as *const uefi::Char16;
    let cstr16 = unsafe { uefi::CStr16::from_ptr(uefiptr) };
    String::from(cstr16)
}

fn ptr_to_guid(ptr: *const Guid) -> uefi::Guid {
    let uefiptr = ptr as *const uefi::Guid;
    unsafe { *uefiptr }
}

pub(crate) fn ptrs_to_id(name: *const Char16, guid: *const Guid) -> EfiVarId {
    EfiVarId::new(ptr_to_string(name), ptr_to_guid(guid))
}

pub(crate) fn id_name_to_cstr16(id: &EfiVarId) -> CString16 {
    CString16::try_from(id.name()).unwrap()
}

pub(crate) fn usize_to_status(status: usize) -> Status {
    unsafe { core::mem::transmute(status) }
}

pub(crate) unsafe fn copy_variable(
    variable: &EfiVar,
    attributes: *mut u32,
    data_size: *mut usize,
    data: *mut (),
) -> Status {
    let outsize = unsafe { *data_size };

    let attr = variable.attr().into();
    let size = variable.data().len();
    unsafe {
        if !attributes.is_null() {
            *attributes = attr;
        }
        *data_size = size;
    };
    if outsize < size {
        return Status::EFI_BUFFER_TOO_SMALL;
    }

    unsafe {
        if !data.is_null() {
            core::ptr::copy(
                variable.data().as_ptr(),
                data as *mut u8,
                variable.data().len(),
            );
        }
    }
    Status::EFI_SUCCESS
}

pub(crate) unsafe fn copy_variable_id(
    variable_id: &EfiVarId,
    variable_name_size: *mut usize,
    variable_name: *mut Char16,
    vendor_guid: *mut Guid,
) -> Status {
    let outsize = unsafe { *variable_name_size };

    let cstr16 = id_name_to_cstr16(variable_id);
    unsafe {
        *variable_name_size = cstr16.num_bytes();
    };
    if outsize < cstr16.num_bytes() {
        return Status::EFI_BUFFER_TOO_SMALL;
    }

    unsafe {
        core::ptr::copy(
            cstr16.as_bytes().as_ptr(),
            variable_name as *mut u8,
            cstr16.num_bytes(),
        );
        let guid_bytes = variable_id.guid().to_bytes();
        core::ptr::copy(
            guid_bytes.as_ptr(),
            vendor_guid as *mut u8,
            guid_bytes.len(),
        );
    }

    Status::EFI_SUCCESS
}
