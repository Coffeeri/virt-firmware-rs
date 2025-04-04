extern crate std;
use std::io::{self, Cursor, Result};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::string::String;
use std::vec;
use std::vec::Vec;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use log::debug;
use uguid::Guid;
use zerocopy::{FromBytes, IntoBytes};

use crate::efivar::boot;
use crate::efivar::devpath as dp;
use crate::nocasestr::NoCaseString;

pub trait ReadEfiExt: io::Read {
    fn get_cursor(&mut self, len: usize) -> Result<Cursor<Vec<u8>>>;
    fn read_blob(&mut self, len: usize) -> Result<Vec<u8>>;
    fn read_ipv4(&mut self) -> Result<Ipv4Addr>;
    fn read_ipv6(&mut self) -> Result<Ipv6Addr>;
    fn read_ascii_string(&mut self) -> Result<String>;
    fn read_efi_string(&mut self) -> Result<String>;
    fn read_efi_guid(&mut self) -> Result<Guid>;
    fn read_boot_index(&mut self) -> Result<boot::BootIndex>;
    fn read_boot_order(&mut self) -> Result<boot::BootOrder>;
    fn read_boot_entry(&mut self) -> Result<boot::BootEntry>;
    fn read_devpath_node(&mut self) -> Result<dp::DevPathNode>;
    fn read_devpath(&mut self) -> Result<dp::DevPath>;
}

pub trait WriteEfiExt: io::Write {
    fn write_ascii_string(&mut self, value: &str) -> Result<()>;
    fn write_efi_string(&mut self, value: &str) -> Result<()>;
    fn write_efi_guid(&mut self, value: &Guid) -> Result<()>;
    fn write_boot_index(&mut self, value: &boot::BootIndex) -> Result<()>;
    fn write_boot_order(&mut self, value: &boot::BootOrder) -> Result<()>;
    fn write_boot_entry(&mut self, value: &boot::BootEntry) -> Result<()>;
    fn write_devpath_node(&mut self, value: &dp::DevPathNode) -> Result<()>;
    fn write_devpath(&mut self, value: &dp::DevPath) -> Result<()>;
}

impl<T: io::Read> ReadEfiExt for T {
    fn get_cursor(&mut self, len: usize) -> Result<Cursor<Vec<u8>>> {
        let blob = self.read_blob(len)?;
        let cursor = Cursor::new(blob);
        Ok(cursor)
    }

    fn read_blob(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut data = vec![0; len];
        self.read_exact(&mut data)?;
        Ok(data)
    }

    fn read_ipv4(&mut self) -> Result<Ipv4Addr> {
        let mut addr_bytes: [u8; 4] = [0; 4];
        self.read_exact(&mut addr_bytes)?;
        Ok(Ipv4Addr::from(addr_bytes))
    }

    fn read_ipv6(&mut self) -> Result<Ipv6Addr> {
        let mut addr_bytes: [u8; 16] = [0; 16];
        self.read_exact(&mut addr_bytes)?;
        Ok(Ipv6Addr::from(addr_bytes))
    }

    fn read_ascii_string(&mut self) -> Result<String> {
        let mut s = String::new();
        loop {
            let u = self.read_u8()?;
            if u == 0 {
                break;
            }
            s.push(u as char);
        }
        Ok(s)
    }

    fn read_efi_string(&mut self) -> Result<String> {
        let mut s = String::new();
        loop {
            let u = self.read_u16::<LE>()?;
            if u == 0 {
                break;
            }
            if let Some(c) = char::from_u32(u as u32) {
                s.push(c);
            }
        }
        Ok(s)
    }

    fn read_efi_guid(&mut self) -> Result<Guid> {
        let mut guid_bytes: [u8; 16] = [0; 16];
        self.read_exact(&mut guid_bytes)?;
        Ok(Guid::from_bytes(guid_bytes))
    }

    fn read_boot_index(&mut self) -> Result<boot::BootIndex> {
        let idx = self.read_u16::<LE>()?;
        Ok(boot::BootIndex(idx))
    }

    fn read_boot_order(&mut self) -> Result<boot::BootOrder> {
        let mut order = Vec::new();
        loop {
            let Ok(idx) = self.read_boot_index() else {
                break;
            };
            order.push(idx);
        }
        Ok(boot::BootOrder(order))
    }

    fn read_boot_entry(&mut self) -> Result<boot::BootEntry> {
        fn try_efi_string(blob: &[u8]) -> Option<String> {
            let mut cursor = Cursor::new(&blob);
            let s = cursor.read_efi_string().ok()?;
            if s.len() * 2 + 2 != blob.len() {
                return None;
            }
            Some(s)
        }

        let attributes = self.read_u32::<LE>()?;
        let devpathlen = self.read_u16::<LE>()?;
        let title = self.read_efi_string()?;

        let mut r = self.get_cursor(devpathlen as usize)?;
        let devpath = r.read_devpath()?;

        let mut optdata_bytes = Vec::new();
        self.read_to_end(&mut optdata_bytes)?;

        let optdata = if optdata_bytes.is_empty() {
            boot::BootEntryOptData::None
        } else if let Some(string) = try_efi_string(&optdata_bytes) {
            boot::BootEntryOptData::String { string }
        } else if optdata_bytes.len() == 16 {
            let bytes: [u8; 16] = optdata_bytes.try_into().unwrap();
            let guid = Guid::from_bytes(bytes);
            boot::BootEntryOptData::Guid { guid }
        } else {
            boot::BootEntryOptData::Data {
                bytes: optdata_bytes,
            }
        };

        Ok(boot::BootEntry {
            attributes,
            title,
            devpath,
            optdata,
        })
    }

    fn read_devpath_node(&mut self) -> Result<dp::DevPathNode> {
        let hdr = dp::DevPathHeader::read_from_io(&mut *self)?;

        match (hdr.devtype, hdr.subtype, (hdr.length - 4) as usize) {
            (dp::TYPE_HW, dp::TYPE_HW_PCI, 2) => {
                let pci = dp::DevPathHwPci::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::HwPci(pci))
            }

            (dp::TYPE_ACPI, dp::TYPE_ACPI_DP, 8) => {
                let acpi = dp::DevPathAcpiDp::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::AcpiDp(acpi))
            }

            (dp::TYPE_ACPI, dp::TYPE_ACPI_ADR, len) => {
                let mut adr = Vec::new();
                for _i in 0..len / 4 {
                    let v = self.read_u32::<LE>()?;
                    adr.push(v);
                }
                Ok(dp::DevPathNode::AcpiAdr(dp::DevPathAcpiAdr { adr }))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_SCSI, 4) => {
                let scsi = dp::DevPathMsgScsi::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::MsgScsi(scsi))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_USB, 2) => {
                let usb = dp::DevPathMsgUsb::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::MsgUsb(usb))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_VENDOR, 16) => {
                let guid = self.read_efi_guid()?;
                Ok(dp::DevPathNode::MsgVendor(dp::DevPathMsgVendor { guid }))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_MAC, 33) => {
                let mac = dp::DevPathMsgMac::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::MsgMac(mac))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_IPV4, 23) => {
                let local_addr = self.read_ipv4()?;
                let remote_addr = self.read_ipv4()?;
                let local_port = self.read_u16::<LE>()?;
                let remote_port = self.read_u16::<LE>()?;
                let protocol = self.read_u16::<LE>()?;
                let static_ip = self.read_u8()?;
                let gw_addr = self.read_ipv4()?;
                let net_mask = self.read_ipv4()?;
                Ok(dp::DevPathNode::MsgIPv4(dp::DevPathMsgIPv4 {
                    local_addr,
                    remote_addr,
                    local_port,
                    remote_port,
                    protocol,
                    static_ip: static_ip != 0,
                    gw_addr,
                    net_mask,
                }))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_IPV6, 56) => {
                let local_addr = self.read_ipv6()?;
                let remote_addr = self.read_ipv6()?;
                let local_port = self.read_u16::<LE>()?;
                let remote_port = self.read_u16::<LE>()?;
                let protocol = self.read_u16::<LE>()?;
                let addr_origin = self.read_u8()?;
                let prefix_len = self.read_u8()?;
                let gw_addr = self.read_ipv6()?;
                Ok(dp::DevPathNode::MsgIPv6(dp::DevPathMsgIPv6 {
                    local_addr,
                    remote_addr,
                    local_port,
                    remote_port,
                    protocol,
                    addr_origin,
                    prefix_len,
                    gw_addr,
                }))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_UART, 15) => {
                let uart = dp::DevPathMsgUart::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::MsgUart(uart))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_SATA, 6) => {
                let sata = dp::DevPathMsgSata::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::MsgSata(sata))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_USB_CLASS, 7) => {
                let class = dp::DevPathMsgUsbClass::read_from_io(&mut *self)?;
                Ok(dp::DevPathNode::MsgUsbClass(class))
            }

            (dp::TYPE_MSG, dp::TYPE_MSG_URI, len) => {
                let uri = if len == 0 {
                    None
                } else {
                    let mut r = self.get_cursor(len)?;
                    Some(r.read_ascii_string()?)
                };
                Ok(dp::DevPathNode::MsgUri(dp::DevPathMsgUri { uri }))
            }

            (dp::TYPE_MEDIA, dp::TYPE_MEDIA_HD, 38) => {
                let nr = self.read_u32::<LE>()?;
                let start = self.read_u64::<LE>()?;
                let size = self.read_u64::<LE>()?;
                let guid = self.read_efi_guid()?;
                let mbrtype = self.read_u8()?;
                let sigtype = self.read_u8()?;
                Ok(dp::DevPathNode::MediaHd {
                    nr,
                    start,
                    size,
                    signature: guid,
                    mbrtype,
                    sigtype,
                })
            }

            (dp::TYPE_MEDIA, dp::TYPE_MEDIA_VENDOR, 16) => {
                let guid = self.read_efi_guid()?;
                Ok(dp::DevPathNode::MediaVendor { guid })
            }

            (dp::TYPE_MEDIA, dp::TYPE_MEDIA_FILEPATH, len) => {
                let mut r = self.get_cursor(len)?;
                let path = r.read_efi_string()?;
                Ok(dp::DevPathNode::MediaFilePath {
                    path: NoCaseString::new(&path),
                })
            }

            (dp::TYPE_MEDIA, dp::TYPE_MEDIA_FW_FILE, 16) => {
                let filename = self.read_efi_guid()?;
                Ok(dp::DevPathNode::MediaFwFile { filename })
            }

            (dp::TYPE_MEDIA, dp::TYPE_MEDIA_FW_VOL, 16) => {
                let volume = self.read_efi_guid()?;
                Ok(dp::DevPathNode::MediaFwVol { volume })
            }

            (dp::TYPE_END, subtype, 0) => Ok(dp::DevPathNode::End { subtype }),

            (d, s, len) => {
                let data = self.read_blob(len)?;
                debug!(
                    "devpath node type=0x{:x} subtype=0x{:x} data={:?}",
                    d, s, data
                );
                Ok(dp::DevPathNode::Other {
                    devtype: d,
                    subtype: s,
                    blob: data,
                })
            }
        }
    }

    fn read_devpath(&mut self) -> Result<dp::DevPath> {
        let mut nodes = Vec::new();
        loop {
            let node = self.read_devpath_node()?;
            match node {
                dp::DevPathNode::End { subtype: _ } => break,
                _ => nodes.push(node),
            }
        }
        Ok(dp::DevPath { nodes })
    }
}

impl<T: io::Write> WriteEfiExt for T {
    fn write_ascii_string(&mut self, value: &str) -> Result<()> {
        for char in value.chars() {
            self.write_u8(char as u8)?;
        }
        self.write_u8(0)
    }

    fn write_efi_string(&mut self, value: &str) -> Result<()> {
        for char in value.chars() {
            self.write_u16::<LE>(char as u16)?;
        }
        self.write_u16::<LE>(0)
    }

    fn write_efi_guid(&mut self, value: &Guid) -> Result<()> {
        self.write_all(&value.to_bytes())
    }

    fn write_boot_index(&mut self, value: &boot::BootIndex) -> Result<()> {
        self.write_all(&Vec::from(value))
    }

    fn write_boot_order(&mut self, value: &boot::BootOrder) -> Result<()> {
        self.write_all(&Vec::from(value))
    }

    fn write_boot_entry(&mut self, value: &boot::BootEntry) -> Result<()> {
        let mut w = Cursor::new(Vec::new());
        w.write_devpath(&value.devpath)?;

        let optdata = match &value.optdata {
            boot::BootEntryOptData::None => [].to_vec(),
            boot::BootEntryOptData::String { string } => {
                let mut cursor = Cursor::new(Vec::new());
                cursor.write_efi_string(string)?;
                cursor.into_inner()
            }
            boot::BootEntryOptData::Guid { guid } => guid.to_bytes().to_vec(),
            boot::BootEntryOptData::Data { bytes } => bytes.to_vec(),
        };

        self.write_u32::<LE>(value.attributes)?;
        self.write_u16::<LE>(w.position() as u16)?;
        self.write_efi_string(&value.title)?;
        self.write_all(&w.into_inner())?;
        self.write_all(&optdata)?;
        Ok(())
    }

    fn write_devpath_node(&mut self, value: &dp::DevPathNode) -> Result<()> {
        match value {
            dp::DevPathNode::HwPci(pci) => {
                let hdr = dp::DevPathHeader::new_hw_pci();
                hdr.write_to_io(&mut *self)?;
                pci.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::AcpiDp(acpi) => {
                let hdr = dp::DevPathHeader::new_acpi_dp();
                hdr.write_to_io(&mut *self)?;
                acpi.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::AcpiAdr(acpi) => {
                let hdr = dp::DevPathHeader::new_acpi_adr(acpi.adr.len());
                hdr.write_to_io(&mut *self)?;
                let adr: &Vec<u32> = acpi.adr.as_ref();
                for i in adr {
                    self.write_u32::<LE>(*i)?;
                }
                Ok(())
            }

            dp::DevPathNode::MsgScsi(scsi) => {
                let hdr = dp::DevPathHeader::new_msg_scsi();
                hdr.write_to_io(&mut *self)?;
                scsi.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::MsgUsb(usb) => {
                let hdr = dp::DevPathHeader::new_msg_usb();
                hdr.write_to_io(&mut *self)?;
                usb.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::MsgVendor(v) => {
                let hdr = dp::DevPathHeader::new_msg_vendor();
                hdr.write_to_io(&mut *self)?;
                self.write_efi_guid(&v.guid)?;
                Ok(())
            }

            dp::DevPathNode::MsgMac(mac) => {
                let hdr = dp::DevPathHeader::new_msg_mac();
                hdr.write_to_io(&mut *self)?;
                mac.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::MsgIPv4(ipv4) => {
                let hdr = dp::DevPathHeader::new_msg_ipv4();
                hdr.write_to_io(&mut *self)?;
                self.write_all(&ipv4.local_addr.octets())?;
                self.write_all(&ipv4.remote_addr.octets())?;
                self.write_u16::<LE>(ipv4.local_port)?;
                self.write_u16::<LE>(ipv4.remote_port)?;
                self.write_u16::<LE>(ipv4.protocol)?;
                self.write_u8(if ipv4.static_ip { 1 } else { 0 })?;
                self.write_all(&ipv4.gw_addr.octets())?;
                self.write_all(&ipv4.net_mask.octets())?;
                Ok(())
            }

            dp::DevPathNode::MsgIPv6(ipv6) => {
                let hdr = dp::DevPathHeader::new_msg_ipv6();
                hdr.write_to_io(&mut *self)?;
                self.write_all(&ipv6.local_addr.octets())?;
                self.write_all(&ipv6.remote_addr.octets())?;
                self.write_u16::<LE>(ipv6.local_port)?;
                self.write_u16::<LE>(ipv6.remote_port)?;
                self.write_u16::<LE>(ipv6.protocol)?;
                self.write_u8(ipv6.addr_origin)?;
                self.write_u8(ipv6.prefix_len)?;
                self.write_all(&ipv6.gw_addr.octets())?;
                Ok(())
            }

            dp::DevPathNode::MsgUart(uart) => {
                let hdr = dp::DevPathHeader::new_msg_uart();
                hdr.write_to_io(&mut *self)?;
                uart.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::MsgSata(sata) => {
                let hdr = dp::DevPathHeader::new_msg_sata();
                hdr.write_to_io(&mut *self)?;
                sata.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::MsgUsbClass(class) => {
                let hdr = dp::DevPathHeader::new_msg_usb_class();
                hdr.write_to_io(&mut *self)?;
                class.write_to_io(&mut *self)?;
                Ok(())
            }

            dp::DevPathNode::MsgUri(u) => {
                self.write_u8(dp::TYPE_MSG)?;
                self.write_u8(dp::TYPE_MSG_URI)?;

                match &u.uri {
                    Some(u) => {
                        let mut w = Cursor::new(Vec::new());
                        w.write_ascii_string(u)?;
                        self.write_u16::<LE>(4 + w.position() as u16)?;
                        self.write_all(&w.into_inner())?;
                    }
                    None => {
                        self.write_u16::<LE>(4)?;
                    }
                }
                Ok(())
            }

            dp::DevPathNode::MediaHd {
                nr,
                start,
                size,
                signature,
                mbrtype,
                sigtype,
            } => {
                self.write_u8(dp::TYPE_MEDIA)?;
                self.write_u8(dp::TYPE_MEDIA_HD)?;
                self.write_u16::<LE>(4 + 38)?;
                self.write_u32::<LE>(*nr)?;
                self.write_u64::<LE>(*start)?;
                self.write_u64::<LE>(*size)?;
                self.write_efi_guid(signature)?;
                self.write_u8(*mbrtype)?;
                self.write_u8(*sigtype)?;
                Ok(())
            }

            dp::DevPathNode::MediaVendor { guid } => {
                self.write_u8(dp::TYPE_MEDIA)?;
                self.write_u8(dp::TYPE_MEDIA_VENDOR)?;
                self.write_u16::<LE>(4 + 16)?;
                self.write_efi_guid(guid)?;
                Ok(())
            }

            dp::DevPathNode::MediaFilePath { path } => {
                let mut w = Cursor::new(Vec::new());
                w.write_efi_string(&path.0)?;

                self.write_u8(dp::TYPE_MEDIA)?;
                self.write_u8(dp::TYPE_MEDIA_FILEPATH)?;
                self.write_u16::<LE>(4 + w.position() as u16)?;
                self.write_all(&w.into_inner())?;
                Ok(())
            }

            dp::DevPathNode::MediaFwFile { filename } => {
                self.write_u8(dp::TYPE_MEDIA)?;
                self.write_u8(dp::TYPE_MEDIA_FW_FILE)?;
                self.write_u16::<LE>(4 + 16)?;
                self.write_efi_guid(filename)?;
                Ok(())
            }

            dp::DevPathNode::MediaFwVol { volume } => {
                self.write_u8(dp::TYPE_MEDIA)?;
                self.write_u8(dp::TYPE_MEDIA_FW_VOL)?;
                self.write_u16::<LE>(4 + 16)?;
                self.write_efi_guid(volume)?;
                Ok(())
            }

            dp::DevPathNode::End { subtype } => {
                self.write_u8(dp::TYPE_END)?;
                self.write_u8(*subtype)?;
                self.write_u16::<LE>(4)?;
                Ok(())
            }

            dp::DevPathNode::Other {
                devtype,
                subtype,
                blob,
            } => {
                self.write_u8(*devtype)?;
                self.write_u8(*subtype)?;
                self.write_u16::<LE>(4 + blob.len() as u16)?;
                self.write_all(blob)?;
                Ok(())
            }
        }
    }

    fn write_devpath(&mut self, value: &dp::DevPath) -> Result<()> {
        for node in &value.nodes {
            self.write_devpath_node(node)?;
        }
        let end = dp::DevPathNode::End { subtype: 0xff };
        self.write_devpath_node(&end)
    }
}
