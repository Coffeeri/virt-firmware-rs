//! efi device paths

extern crate std;
use std::fmt;
use std::format;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::string::String;
use std::vec::Vec;

use uuid::Uuid;

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

#[derive(Debug, PartialEq)]
pub enum DevPathNode {
    HwPci {
        function: u8,
        device: u8,
    },

    AcpiDp {
        hid: u32,
        uid: u32,
    },
    AcpiAdr {
        adr: Vec<u32>,
    },

    MsgScsi {
        target: u16,
        lun: u16,
    },
    MsgUsb {
        port: u8,
        interface: u8,
    },
    MsgVendor {
        guid: Uuid,
    },
    MsgMac {
        mac: [u8; 32],
        iftype: u8,
    },
    MsgIPv4 {
        local_addr: Ipv4Addr,
        remote_addr: Ipv4Addr,
        local_port: u16,
        remote_port: u16,
        protocol: u16,
        static_ip: bool,
        gw_addr: Ipv4Addr,
        net_mask: Ipv4Addr,
    },
    MsgIPv6 {
        local_addr: Ipv6Addr,
        remote_addr: Ipv6Addr,
        local_port: u16,
        remote_port: u16,
        protocol: u16,
        addr_origin: u8,
        prefix_len: u8,
        gw_addr: Ipv6Addr,
    },
    MsgUart {
        reserved: u32,
        baudrate: u64,
        databits: u8,
        parity: u8,
        stopbits: u8,
    },
    MsgSata {
        port: u16,
        mul_port: u16,
        lun: u16,
    },
    MsgUsbClass {
        vendor: u16,
        product: u16,
        class: u8,
        subclass: u8,
        protocol: u8,
    },
    MsgUri {
        uri: Option<String>,
    },

    MediaHd {
        nr: u32,
        start: u64,
        size: u64,
        signature: Uuid,
        mbrtype: u8, // use enum ?
        sigtype: u8, // use enum ?
    },
    MediaVendor {
        guid: Uuid,
    },
    MediaFilePath {
        path: NoCaseString,
    },
    MediaFwFile {
        filename: Uuid,
    },
    MediaFwVol {
        volume: Uuid,
    },

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
            DevPathNode::HwPci { device, function } => {
                write!(f, "Pci({:02x}.{:x})", device, function)
            }

            DevPathNode::AcpiDp { hid, uid } => match (hid, uid) {
                (0xa0341d0, _) => {
                    write!(f, "PciRoot()")
                }
                (h, u) => {
                    write!(f, "Acpi(hid=0x{:x},uid=0x{:x})", h, u)
                }
            },

            DevPathNode::AcpiAdr { adr } => {
                let s: Vec<String> = adr.iter().map(|a| format!("0x{:x}", a)).collect();
                write!(f, "Acpi(adr={})", s.join(","))
            }

            DevPathNode::MsgScsi { target, lun } => {
                write!(f, "Scsi(target={},lun={})", target, lun)
            }

            DevPathNode::MsgUsb { port, interface } => {
                write!(f, "Usb(port={},if={})", port, interface)
            }

            DevPathNode::MsgVendor { guid } => {
                write!(f, "Vendor({})", guid)
            }

            DevPathNode::MsgMac { mac: _, iftype: _ } => {
                write!(f, "MAC()")
            }

            DevPathNode::MsgIPv4 {
                local_addr: _,
                remote_addr: _,
                local_port: _,
                remote_port: _,
                protocol: _,
                static_ip,
                gw_addr: _,
                net_mask: _,
            } => {
                if *static_ip {
                    write!(f, "IPv4(static)")
                } else {
                    write!(f, "IPv4(dhcp)")
                }
            }

            DevPathNode::MsgIPv6 {
                local_addr: _,
                remote_addr: _,
                local_port: _,
                remote_port: _,
                protocol: _,
                addr_origin,
                prefix_len: _,
                gw_addr: _,
            } => match addr_origin {
                0 => write!(f, "IPv6(static)"),
                1 => write!(f, "IPv6(stateless)"),
                2 => write!(f, "IPv6(stateful)"),
                _ => write!(f, "IPv6(invalid)"),
            },

            DevPathNode::MsgUart {
                reserved: _,
                baudrate: _,
                databits: _,
                parity: _,
                stopbits: _,
            } => {
                write!(f, "UART()")
            }

            DevPathNode::MsgSata {
                port,
                mul_port: _,
                lun: _,
            } => {
                write!(f, "SATA(port={})", port)
            }

            DevPathNode::MsgUsbClass {
                vendor: _,
                product: _,
                class: _,
                subclass: _s,
                protocol: _,
            } => {
                write!(f, "UsbClass()")
            }
            DevPathNode::MsgUri { uri } => match uri {
                None => write!(f, "URI()"),
                Some(u) => write!(f, "URI(\"{}\")", u),
            },

            DevPathNode::MediaHd {
                nr,
                start: _,
                size: _,
                signature: _,
                mbrtype: _,
                sigtype: _,
            } => {
                write!(f, "HD({})", nr)
            }

            DevPathNode::MediaVendor { guid } => {
                write!(f, "Vendor({})", guid)
            }

            DevPathNode::MediaFilePath { path } => {
                write!(f, "File(\"{}\")", path)
            }

            DevPathNode::MediaFwFile { filename } => {
                write!(f, "FwFile({})", filename)
            }

            DevPathNode::MediaFwVol { volume } => {
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
