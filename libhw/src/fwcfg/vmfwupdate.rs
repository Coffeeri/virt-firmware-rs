use bitfield_struct::bitfield;
use core::mem::size_of;
use log::debug;
use num_enum::FromPrimitive;
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::fwcfg::FwCfg;

pub const VMFWUPDATE_NAME: &str = "etc/vmlaunchupdate";

pub const VMFWUPDATE_VERSION: u16 = 1;

#[bitfield(u64)]
pub struct VmFwUpdateCaps {
    #[bits(32)]
    _reserved1: u32,

    pub format_igvm: bool,
    #[bits(31)]
    _reserved2: u32,
}

#[bitfield(u64)]
pub struct VmFwUpdateCtrl {
    pub ctrl_disable: bool,
    pub ctrl_once: bool,
    #[bits(30)]
    _reserved1: u32,

    pub format_igvm: bool,
    #[bits(31)]
    _reserved2: u32,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive)]
#[repr(u16)]
pub enum VmFwUpdateStatus {
    Success = 0,
    LoadFail = 1,
    IgvmInitFail = 2,
    #[num_enum(default)]
    UnknownFail = 0xffff,
}

#[derive(Debug, Default, FromBytes, IntoBytes, Immutable)]
#[repr(C, packed)]
pub struct VmFwUpdate {
    pub version: u16,
    pub status: u16, // VmFwUpdateStatus,
    _padding: [u8; 4],
    pub capabilities: u64, // VmFwUpdateCaps,
    pub control: u64,      // VmFwUpdateCtrl,
    pub firmware_image_addr: u64,
    pub firmware_image_size: u64,
    pub opaque_blob_addr: u64,
    pub opaque_blob_size: u64,
}

pub struct FirmwareLoader<'f> {
    fwcfg: &'f dyn FwCfg,
    item: u16,
    size: u32,
    update: VmFwUpdate,
}

impl<'f> FirmwareLoader<'f> {
    pub fn new(fwcfg: &'f dyn FwCfg) -> Option<Self> {
        let (item, size) = fwcfg.findfile(VMFWUPDATE_NAME)?;
        debug!("vmfwupdate: file ok (item {item}, size {size})");

        if size as usize != size_of::<VmFwUpdate>() {
            return None;
        }
        debug!("vmfwupdate: size ok");

        let update = VmFwUpdate::default();
        let mut loader = Self {
            fwcfg,
            item,
            size,
            update,
        };
        loader.read_update();

        Some(loader)
    }

    fn read_update(&mut self) {
        self.fwcfg.read_dma(
            Some(self.item),
            self.update.as_mut_bytes().as_mut_ptr().cast(),
            self.size as usize,
        );
    }

    fn write_update(&mut self) {
        self.fwcfg.write_dma(
            self.item,
            self.update.as_mut_bytes().as_mut_ptr().cast(),
            self.size as usize,
        );
    }

    pub fn get_version(&self) -> u16 {
        self.update.version
    }

    pub fn get_caps(&self) -> VmFwUpdateCaps {
        self.update.capabilities.into()
    }

    pub fn set_igvm(&mut self, igvm: &[u8], once: bool) -> VmFwUpdateStatus {
        let ptr = igvm.as_ptr();
        self.update.firmware_image_addr = ptr.addr() as u64;
        self.update.firmware_image_size = igvm.len() as u64;
        self.update.control = VmFwUpdateCtrl::new()
            .with_format_igvm(true)
            .with_ctrl_once(once)
            .into();
        self.write_update();
        self.read_update();
        self.update.status.into()
    }
}
