#![cfg(feature = "std")]
use std::io::Cursor;

use virtfw_libefi::efivar::io::ReadEfiExt;
use virtfw_libefi::efivar::io::WriteEfiExt;

const DATA_EMPTY: [u8; 0] = [];
const DATA_BOOTORDER_2: &[u8] = include_bytes!("data/bootorder_2");
const DATA_BOOTENTRY_USB: &[u8] = include_bytes!("data/bootentry_usb");
const DATA_BOOTENTRY_SATA: &[u8] = include_bytes!("data/bootentry_sata");
const DATA_BOOTENTRY_SCSI: &[u8] = include_bytes!("data/bootentry_scsi");
const DATA_BOOTENTRY_PXE_V4: &[u8] = include_bytes!("data/bootentry_pxe_v4");
const DATA_BOOTENTRY_HTTP_V6: &[u8] = include_bytes!("data/bootentry_http_v6");
const DATA_BOOTENTRY_RHEL_GRUB: &[u8] = include_bytes!("data/bootentry_rhel_grub");
const DATA_BOOTENTRY_FWSETUP: &[u8] = include_bytes!("data/bootentry_fwsetup");

#[test]
fn bootorder_empty() {
    let mut rd = Cursor::new(DATA_EMPTY);
    let order = rd.read_boot_order().unwrap();
    assert!(order.0.is_empty())
}

#[test]
fn bootorder_2() {
    let mut rd = Cursor::new(DATA_BOOTORDER_2);
    let order = rd.read_boot_order().unwrap();
    assert_eq!(format!("{}", order), "0001,0002");
    let bytes = Vec::from(&order);
    assert_eq!(bytes, DATA_BOOTORDER_2);
}

#[test]
fn bootentry_usb() {
    let mut rd = Cursor::new(DATA_BOOTENTRY_USB);
    let entry = rd.read_boot_entry().unwrap();
    assert_eq!(
        format!("{}", entry.devpath),
        "PciRoot()/Pci(02.1)/Pci(00.0)/Usb(port=1,if=0)"
    );
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry).unwrap();
    assert_eq!(wr.into_inner(), DATA_BOOTENTRY_USB);
}

#[test]
fn bootentry_sata() {
    let mut rd = Cursor::new(DATA_BOOTENTRY_SATA);
    let entry = rd.read_boot_entry().unwrap();
    assert_eq!(
        format!("{}", entry.devpath),
        "PciRoot()/Pci(1f.2)/SATA(port=1)"
    );
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry).unwrap();
    assert_eq!(wr.into_inner(), DATA_BOOTENTRY_SATA);
}

#[test]
fn bootentry_scsi() {
    let mut rd = Cursor::new(DATA_BOOTENTRY_SCSI);
    let entry = rd.read_boot_entry().unwrap();
    assert_eq!(
        format!("{}", entry.devpath),
        "PciRoot()/Pci(02.2)/Pci(00.0)/Scsi(target=0,lun=0)"
    );
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry).unwrap();
    assert_eq!(wr.into_inner(), DATA_BOOTENTRY_SCSI);
}

#[test]
fn bootentry_pxe_v4() {
    let mut rd = Cursor::new(DATA_BOOTENTRY_PXE_V4);
    let entry = rd.read_boot_entry().unwrap();
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry).unwrap();
    assert_eq!(wr.into_inner(), DATA_BOOTENTRY_PXE_V4);
}

#[test]
fn bootentry_http_v6() {
    let mut rd = Cursor::new(DATA_BOOTENTRY_HTTP_V6);
    let entry = rd.read_boot_entry().unwrap();
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry).unwrap();
    assert_eq!(wr.into_inner(), DATA_BOOTENTRY_HTTP_V6);
}

#[test]
fn bootentry_rhel_grub() {
    let mut rd = Cursor::new(DATA_BOOTENTRY_RHEL_GRUB);
    let entry = rd.read_boot_entry().unwrap();
    assert_eq!(
        format!("{}", entry.devpath),
        "HD(1)/File(\"\\EFI\\redhat\\shimx64.efi\")"
    );
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry).unwrap();
    assert_eq!(wr.into_inner(), DATA_BOOTENTRY_RHEL_GRUB);
}

#[test]
fn bootentry_fwsetup() {
    let mut rd = Cursor::new(DATA_BOOTENTRY_FWSETUP);
    let entry = rd.read_boot_entry().unwrap();
    assert_eq!(
        format!("{}", entry.devpath),
        "FwVol(7cb8bdc9-f8eb-4f34-aaea-3ee4af6516a1)/FwFile(462caa21-7614-4503-836e-8ab6f4662331)"
    );
    let mut wr = Cursor::new(Vec::new());
    wr.write_boot_entry(&entry).unwrap();
    assert_eq!(wr.into_inner(), DATA_BOOTENTRY_FWSETUP);
}
