//
// persient variable store implemented by qemu (aka -device uefi-vars-x64).
//
extern crate alloc;
use alloc::vec::Vec;
use core::mem::size_of;
use log::{debug, info};
use spin::Mutex;
use zerocopy::{FromBytes, IntoBytes};

use efiloader::status::*;
use efiloader::{Char16, EfiContext, Guid};

use virtfw_libefi::efivar::types::{EfiVar, EfiVarId};
use virtfw_libefi::guids;
use virtfw_libhw::fwcfg::x86::FwCfgX86;
use virtfw_libhw::fwcfg::FwCfg;
use virtfw_libhw::mmio;
use virtfw_varstore::mm::core::MmCoreHeader;
use virtfw_varstore::mm::variable::{MmVariableHeader, MmVariableParsedRequest};

use crate::efi::vars::util::{copy_variable, copy_variable_id, ptrs_to_id, usize_to_status};

struct UefiVars {
    mmio: Option<u64>,
}

#[allow(unused)]
impl UefiVars {
    // register offsets
    const REG_MAGIC: u64 = 0x00; // 16 bit
    const REG_CMD_STS: u64 = 0x02; // 16 bit
    const REG_BUFFER_SIZE: u64 = 0x04; // 32 bit
    const REG_DMA_BUFFER_ADDR_LO: u64 = 0x08; // 32 bi
    const REG_DMA_BUFFER_ADDR_HI: u64 = 0x0c; // 32 bit
    const REG_PIO_BUFFER_TRANSFER: u64 = 0x10; // 8-64 bit
    const REG_PIO_BUFFER_CRC32C: u64 = 0x18; // 32 bit (read-only)
    const REG_FLAGS: u64 = 0x1c; // 32 bit
    const REGS_SIZE: u64 = 0x20;

    // flags register
    const FLAG_USE_PIO: u32 = (1 << 0);

    // magic value
    const MAGIC_VALUE: u16 = 0xef1;

    // command values
    const CMD_RESET: u16 = 0x01;
    const CMD_DMA_MM: u16 = 0x02;
    const CMD_PIO_MM: u16 = 0x03;
    const CMD_PIO_ZERO_OFFSET: u16 = 0x04;

    // status values
    const STS_SUCCESS: u16 = 0x00;
    const STS_BUSY: u16 = 0x01;
    const STS_ERR_UNKNOWN: u16 = 0x10;
    const STS_ERR_NOT_SUPPORTED: u16 = 0x11;
    const STS_ERR_BAD_BUFFER_SIZE: u16 = 0x12;

    const fn new() -> Self {
        Self { mmio: None }
    }

    fn init(&mut self, mmio: u64, buffer: &[u8]) -> Result<(), &'static str> {
        let magic = mmio::readw(mmio + Self::REG_MAGIC);
        if magic != Self::MAGIC_VALUE {
            return Err("uefi-vars: magic mismatch");
        }

        let addr_hi = ((buffer.as_ptr() as usize >> 32) & 0xffffffff) as u32;
        let addr_lo = ((buffer.as_ptr() as usize) & 0xffffffff) as u32;

        info!(
            "uefi-vars: buffer addr/hi 0x{:x}, addr/lo 0x{:x}, size 0x{:x}",
            addr_hi,
            addr_lo,
            buffer.len()
        );

        mmio::writew(mmio + Self::REG_CMD_STS, Self::CMD_RESET);
        mmio::writel(mmio + Self::REG_DMA_BUFFER_ADDR_HI, addr_hi);
        mmio::writel(mmio + Self::REG_DMA_BUFFER_ADDR_LO, addr_lo);
        mmio::writel(mmio + Self::REG_BUFFER_SIZE, buffer.len() as u32);

        self.mmio = Some(mmio);
        Ok(())
    }

    fn dma_mm(&self) -> Result<(), &'static str> {
        let Some(mmio) = self.mmio else {
            return Err("uefi-vars: not initialized");
        };

        mmio::writew(mmio + Self::REG_CMD_STS, Self::CMD_DMA_MM);
        loop {
            let status = mmio::readw(mmio + Self::REG_CMD_STS);
            match status {
                Self::STS_SUCCESS => return Ok(()),
                Self::STS_BUSY => continue,
                _ => return Err("uefi-vars: mm command failed"),
            }
        }
    }
}

#[repr(align(4096))]
struct RequestBuffer([u8; 64 * 1024]);

impl RequestBuffer {
    const fn new() -> Self {
        Self([0; 64 * 1024])
    }

    fn serialize(&mut self, req: &MmVariableParsedRequest) {
        let vars_buf = Vec::<u8>::try_from(req).unwrap();
        let core_hdr = MmCoreHeader::new(guids::EfiSmmVariableProtocol, vars_buf.len() as u64);
        let offset = size_of::<MmCoreHeader>();
        self.0[0..offset].copy_from_slice(core_hdr.as_bytes());
        self.0[offset..offset + vars_buf.len()].copy_from_slice(&vars_buf);
    }

    fn result_variable(&self) -> Result<EfiVar, Status> {
        let offset = size_of::<MmCoreHeader>();
        let (header, body) = MmVariableHeader::from_data(&self.0[offset..]).unwrap();
        match header.status {
            0 => {
                if let Ok(var) = MmVariableParsedRequest::parse_access_var(body) {
                    Ok(var)
                } else {
                    Err(Status::EFI_DEVICE_ERROR)
                }
            }
            st => Err(usize_to_status(st as usize)),
        }
    }

    fn result_variable_next(&self) -> Result<EfiVarId, Status> {
        let offset = size_of::<MmCoreHeader>();
        let (header, body) = MmVariableHeader::from_data(&self.0[offset..]).unwrap();
        match header.status {
            0 => {
                if let Ok(id) = MmVariableParsedRequest::parse_next(body) {
                    Ok(id)
                } else {
                    Err(Status::EFI_DEVICE_ERROR)
                }
            }
            st => Err(usize_to_status(st as usize)),
        }
    }

    fn result_status(&self) -> Status {
        let offset = size_of::<MmCoreHeader>();
        let (header, _body) = MmVariableHeader::from_data(&self.0[offset..]).unwrap();
        usize_to_status(header.status as usize)
    }
}

static UEFIVARS: Mutex<UefiVars> = Mutex::new(UefiVars::new());
static REQBUF: Mutex<RequestBuffer> = Mutex::new(RequestBuffer::new());

extern "efiapi" fn qemuvars_get_variable(
    variable_name: *const Char16,
    vendor_guid: *const Guid,
    attributes: *mut u32,
    data_size: *mut usize,
    data: *mut (),
) -> Status {
    if variable_name.is_null() || vendor_guid.is_null() || data_size.is_null() {
        return Status::EFI_INVALID_PARAMETER;
    }

    let id = ptrs_to_id(variable_name, vendor_guid);
    info!("get-variable: {}", id.name());

    let req = MmVariableParsedRequest::GetVariable(id);
    let mut reqbuf = REQBUF.lock();
    reqbuf.serialize(&req);

    let uefivars = UEFIVARS.lock();
    if let Err(_err) = uefivars.dma_mm() {
        return Status::EFI_DEVICE_ERROR;
    }

    let result = reqbuf.result_variable();
    if let Err(status) = result {
        return status;
    }
    let var = result.unwrap();

    unsafe { copy_variable(&var, attributes, data_size, data) }
}

extern "efiapi" fn qemuvars_get_next_variable_name(
    variable_name_size: *mut usize,
    variable_name: *mut Char16,
    vendor_guid: *mut Guid,
) -> Status {
    if variable_name_size.is_null() || variable_name.is_null() || vendor_guid.is_null() {
        return Status::EFI_INVALID_PARAMETER;
    }

    let id = ptrs_to_id(variable_name, vendor_guid);

    let req = MmVariableParsedRequest::GetNextVariableName(id);
    let mut reqbuf = REQBUF.lock();
    reqbuf.serialize(&req);

    let uefivars = UEFIVARS.lock();
    if let Err(_err) = uefivars.dma_mm() {
        return Status::EFI_DEVICE_ERROR;
    }

    let result = reqbuf.result_variable_next();
    if let Err(status) = result {
        return status;
    }
    let next_id = result.unwrap();

    unsafe { copy_variable_id(&next_id, variable_name_size, variable_name, vendor_guid) }
}

extern "efiapi" fn qemuvars_set_valiable(
    variable_name: *const Char16,
    vendor_guid: *const Guid,
    attributes: u32,
    data_size: usize,
    data: *const (),
) -> Status {
    if variable_name.is_null() || vendor_guid.is_null() || data.is_null() {
        return Status::EFI_INVALID_PARAMETER;
    }

    let id = ptrs_to_id(variable_name, vendor_guid);
    info!("set-variable: {}", id.name());
    let data = unsafe { core::slice::from_raw_parts(data as *const u8, data_size) };
    let var = EfiVar::new_with_vec(id, attributes.into(), data.to_vec());

    let req = MmVariableParsedRequest::SetVariable(var);
    let mut reqbuf = REQBUF.lock();
    reqbuf.serialize(&req);

    let uefivars = UEFIVARS.lock();
    if let Err(_err) = uefivars.dma_mm() {
        return Status::EFI_DEVICE_ERROR;
    }

    reqbuf.result_status()
}

#[derive(Debug, FromBytes)]
struct HardwareInfoHdr {
    hwtype: u64,
    size: u64,
}

pub fn probe(fwcfg: &FwCfgX86) -> Result<u64, &'static str> {
    let Some(hwinfo) = fwcfg.load_file("etc/hardware-info") else {
        return Err("hwinfo: etc/hardware-info not present");
    };
    let mut data = hwinfo.as_ref();
    loop {
        let result = HardwareInfoHdr::read_from_prefix(data);
        if result.is_err() {
            return Err("hwinfo: header parse error");
        }
        let (hdr, rest) = result.unwrap();
        debug!("hwinfo: {hdr:?}");
        if hdr.hwtype == 2 {
            /* HardwareInfoQemuUefiVars */
            let result = u64::read_from_prefix(rest);
            if result.is_err() {
                return Err("hwinfo: mmio parse error");
            }
            let (mmio, _) = result.unwrap();
            return Ok(mmio);
        }
        if rest.len() <= hdr.size as usize {
            return Err("hwinfo: uefi vars not found");
        }
        data = &rest[(hdr.size as usize)..];
    }
}

pub fn init(efi: &EfiContext, fwcfg: &FwCfgX86) -> Result<(), &'static str> {
    let mmio = probe(fwcfg)?;
    info!("hwinfo: uefi-vars-x64 at 0x{mmio:x}");

    let reqbuf = REQBUF.lock();
    let mut uefivars = UEFIVARS.lock();
    uefivars.init(mmio, &reqbuf.0)?;

    efi.override_variable_handler(
        qemuvars_get_variable,
        qemuvars_get_next_variable_name,
        Some(qemuvars_set_valiable),
    );
    Ok(())
}
