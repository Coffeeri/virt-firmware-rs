//! efi device paths

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use core::fmt;
use core::mem::size_of;
use core::net::{Ipv4Addr, Ipv6Addr};

use uguid::Guid;
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};

use crate::nocasestr::NoCaseString;

pub const TYPE_HW: u8 = 1;
pub const TYPE_ACPI: u8 = 2;
pub const TYPE_MSG: u8 = 3;
pub const TYPE_MEDIA: u8 = 4;
pub const TYPE_BBS: u8 = 5;
pub const TYPE_END: u8 = 0x7f;

pub const TYPE_HW_PCI: u8 = 1;

pub const TYPE_ACPI_DP: u8 = 1;
pub const TYPE_ACPI_DPX: u8 = 2;
pub const TYPE_ACPI_ADR: u8 = 3;

pub const TYPE_MSG_SCSI: u8 = 2;
pub const TYPE_MSG_USB: u8 = 5;
pub const TYPE_MSG_VENDOR: u8 = 0x0a;
pub const TYPE_MSG_MAC: u8 = 0x0b;
pub const TYPE_MSG_IPV4: u8 = 0x0c;
pub const TYPE_MSG_IPV6: u8 = 0x0d;
pub const TYPE_MSG_UART: u8 = 0x0e;
pub const TYPE_MSG_USB_CLASS: u8 = 0x0f;
pub const TYPE_MSG_SATA: u8 = 0x12;
pub const TYPE_MSG_URI: u8 = 0x18;

pub const TYPE_MEDIA_HD: u8 = 1;
pub const TYPE_MEDIA_CD: u8 = 2;
pub const TYPE_MEDIA_VENDOR: u8 = 3;
pub const TYPE_MEDIA_FILEPATH: u8 = 4;
pub const TYPE_MEDIA_PROTOCOL: u8 = 5;
pub const TYPE_MEDIA_FW_FILE: u8 = 6;
pub const TYPE_MEDIA_FW_VOL: u8 = 7;

#[repr(C)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathHeader {
    pub devtype: u8,
    pub subtype: u8,
    pub length: u16,
}

impl DevPathHeader {
    pub fn new(devtype: u8, subtype: u8, length: usize) -> Self {
        DevPathHeader {
            devtype,
            subtype,
            length: (size_of::<DevPathHeader>() + length) as u16,
        }
    }

    pub fn new_hw_pci() -> Self {
        DevPathHeader::new(TYPE_HW, TYPE_HW_PCI, size_of::<DevPathHwPci>())
    }

    pub fn new_acpi_dp() -> Self {
        DevPathHeader::new(TYPE_ACPI, TYPE_ACPI_DP, size_of::<DevPathAcpiDp>())
    }

    pub fn new_acpi_adr(elems: usize) -> Self {
        DevPathHeader::new(TYPE_ACPI, TYPE_ACPI_ADR, elems * size_of::<u32>())
    }

    pub fn new_msg_scsi() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_SCSI, size_of::<DevPathMsgScsi>())
    }

    pub fn new_msg_usb() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_USB, size_of::<DevPathMsgUsb>())
    }

    pub fn new_msg_vendor() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_VENDOR, size_of::<Guid>())
    }

    pub fn new_msg_mac() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_MAC, size_of::<DevPathMsgMac>())
    }

    pub fn new_msg_ipv4() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_IPV4, 23)
    }

    pub fn new_msg_ipv6() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_IPV6, 56)
    }

    pub fn new_msg_uart() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_UART, size_of::<DevPathMsgUart>())
    }

    pub fn new_msg_sata() -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_SATA, size_of::<DevPathMsgSata>())
    }

    pub fn new_msg_usb_class() -> Self {
        DevPathHeader::new(
            TYPE_MSG,
            TYPE_MSG_USB_CLASS,
            size_of::<DevPathMsgUsbClass>(),
        )
    }

    pub fn new_msg_uri(bytes: usize) -> Self {
        DevPathHeader::new(TYPE_MSG, TYPE_MSG_URI, bytes)
    }

    pub fn new_media_hd() -> Self {
        DevPathHeader::new(TYPE_MEDIA, TYPE_MEDIA_HD, 38)
    }

    pub fn new_media_vendor() -> Self {
        DevPathHeader::new(TYPE_MEDIA, TYPE_MEDIA_VENDOR, size_of::<Guid>())
    }

    pub fn new_media_fw_file() -> Self {
        DevPathHeader::new(TYPE_MEDIA, TYPE_MEDIA_FW_FILE, size_of::<Guid>())
    }

    pub fn new_media_fw_vol() -> Self {
        DevPathHeader::new(TYPE_MEDIA, TYPE_MEDIA_FW_VOL, size_of::<Guid>())
    }

    pub fn new_media_filepath(bytes: usize) -> Self {
        DevPathHeader::new(TYPE_MEDIA, TYPE_MEDIA_FILEPATH, bytes)
    }

    pub fn new_end(subtype: u8) -> Self {
        DevPathHeader::new(TYPE_END, subtype, 0)
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathHwPci {
    pub function: u8,
    pub device: u8,
}

#[repr(C)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathAcpiDp {
    pub hid: u32,
    pub uid: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathMsgScsi {
    pub target: u16,
    pub lun: u16,
}

#[repr(C)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathMsgUsb {
    pub port: u8,
    pub interface: u8,
}

#[repr(C)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathMsgMac {
    pub mac: [u8; 32],
    pub iftype: u8,
}

#[derive(Debug, PartialEq)]
pub struct DevPathMsgIPv4 {
    pub local_addr: Ipv4Addr,
    pub remote_addr: Ipv4Addr,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: u16,
    pub static_ip: bool,
    pub gw_addr: Ipv4Addr,
    pub net_mask: Ipv4Addr,
}

#[derive(Debug, PartialEq)]
pub struct DevPathMsgIPv6 {
    pub local_addr: Ipv6Addr,
    pub remote_addr: Ipv6Addr,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: u16,
    pub addr_origin: u8,
    pub prefix_len: u8,
    pub gw_addr: Ipv6Addr,
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathMsgUart {
    pub reserved: u32,
    pub baudrate: u64,
    pub databits: u8,
    pub parity: u8,
    pub stopbits: u8,
}

#[repr(C)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathMsgSata {
    pub port: u16,
    pub mul_port: u16,
    pub lun: u16,
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct DevPathMsgUsbClass {
    pub vendor: u16,
    pub product: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
}

#[derive(Debug, PartialEq)]
pub struct DevPathMediaHd {
    pub nr: u32,
    pub start: u64,
    pub size: u64,
    pub signature: Guid,
    pub mbrtype: u8, // use enum ?
    pub sigtype: u8, // use enum ?
}

#[derive(Debug, PartialEq)]
pub enum DevPathNode {
    HwPci(DevPathHwPci),

    AcpiDp(DevPathAcpiDp),
    AcpiAdr(Vec<u32>),

    MsgScsi(DevPathMsgScsi),
    MsgUsb(DevPathMsgUsb),
    MsgVendor(Guid),
    MsgMac(DevPathMsgMac),
    MsgIPv4(DevPathMsgIPv4),
    MsgIPv6(DevPathMsgIPv6),
    MsgUart(DevPathMsgUart),
    MsgSata(DevPathMsgSata),
    MsgUsbClass(DevPathMsgUsbClass),
    MsgUri(Option<String>),

    MediaHd(DevPathMediaHd),
    MediaVendor(Guid),
    MediaFilePath(NoCaseString),
    MediaFwFile(Guid),
    MediaFwVol(Guid),

    End {
        subtype: u8,
    },

    // generic catch for unimplemented types
    Other {
        devtype: u8,
        subtype: u8,
        blob: Vec<u8>,
    },
}

impl fmt::Display for DevPathNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DevPathNode::HwPci(pci) => {
                write!(f, "Pci({:02x}.{:x})", pci.device, pci.function)
            }

            DevPathNode::AcpiDp(acpi) => match (acpi.hid, acpi.uid) {
                (0xa0341d0, _) => {
                    write!(f, "PciRoot()")
                }
                (h, u) => {
                    write!(f, "Acpi(hid=0x{:x},uid=0x{:x})", h, u)
                }
            },

            DevPathNode::AcpiAdr(adr) => {
                let s: Vec<String> = adr.iter().map(|a| format!("0x{:x}", a)).collect();
                write!(f, "Acpi(adr={})", s.join(","))
            }

            DevPathNode::MsgScsi(scsi) => {
                write!(f, "Scsi(target={},lun={})", scsi.target, scsi.lun)
            }

            DevPathNode::MsgUsb(usb) => {
                write!(f, "Usb(port={},if={})", usb.port, usb.interface)
            }

            DevPathNode::MsgVendor(guid) => {
                write!(f, "Vendor({})", guid)
            }

            DevPathNode::MsgMac(_) => {
                write!(f, "MAC()")
            }

            DevPathNode::MsgIPv4(ipv4) => {
                if ipv4.static_ip {
                    write!(f, "IPv4(static)")
                } else {
                    write!(f, "IPv4(dhcp)")
                }
            }

            DevPathNode::MsgIPv6(ipv6) => match ipv6.addr_origin {
                0 => write!(f, "IPv6(static)"),
                1 => write!(f, "IPv6(stateless)"),
                2 => write!(f, "IPv6(stateful)"),
                _ => write!(f, "IPv6(invalid)"),
            },

            DevPathNode::MsgUart(_) => {
                write!(f, "UART()")
            }

            DevPathNode::MsgSata(sata) => {
                write!(f, "SATA(port={})", sata.port)
            }

            DevPathNode::MsgUsbClass(_) => {
                write!(f, "UsbClass()")
            }
            DevPathNode::MsgUri(uri) => match &uri {
                None => write!(f, "URI()"),
                Some(u) => write!(f, "URI(\"{}\")", u),
            },

            DevPathNode::MediaHd(part) => {
                write!(f, "HD({})", part.nr)
            }

            DevPathNode::MediaVendor(guid) => {
                write!(f, "Vendor({})", guid)
            }

            DevPathNode::MediaFilePath(path) => {
                write!(f, "File(\"{}\")", path)
            }

            DevPathNode::MediaFwFile(filename) => {
                write!(f, "FwFile({})", filename)
            }

            DevPathNode::MediaFwVol(volume) => {
                write!(f, "FwVol({})", volume)
            }

            DevPathNode::End { subtype: _ } => {
                write!(f, "End()")
            }

            DevPathNode::Other {
                devtype,
                subtype,
                blob,
            } => {
                write!(
                    f,
                    "Other(type=0x{:02x}.0x{:02x},len={})",
                    devtype,
                    subtype,
                    blob.len()
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DevPath {
    pub nodes: Vec<DevPathNode>,
}

impl DevPath {
    pub fn new() -> DevPath {
        let nodes = Vec::new();
        DevPath { nodes }
    }
    pub fn push(&mut self, node: DevPathNode) {
        self.nodes.push(node);
    }
}

impl Default for DevPath {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DevPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: Vec<String> = self.nodes.iter().map(|a| format!("{}", a)).collect();
        write!(f, "{}", s.join("/"))
    }
}

#[test]
fn test_struct_sizes() {
    use core::mem::size_of;

    assert_eq!(size_of::<Guid>(), 16);

    assert_eq!(size_of::<DevPathHwPci>(), 2);
    assert_eq!(size_of::<DevPathAcpiDp>(), 8);

    assert_eq!(size_of::<DevPathMsgScsi>(), 4);
    assert_eq!(size_of::<DevPathMsgUsb>(), 2);
    assert_eq!(size_of::<DevPathMsgMac>(), 33);
    assert_eq!(size_of::<DevPathMsgSata>(), 6);
    assert_eq!(size_of::<DevPathMsgUart>(), 15);
    assert_eq!(size_of::<DevPathMsgUsbClass>(), 7);
}
