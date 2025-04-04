extern crate std;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::string::{String, ToString};

use log::error;
use uguid::Guid;

use crate::efivar::devpath;
use crate::nocasestr::NoCaseString;

pub struct EfiFile {
    pub file: PathBuf,
    pub fdev: u64,
    pub mount: PathBuf,
    pub rfile: PathBuf,
    pub device: udev::Device,
}

impl EfiFile {
    fn mount(file: &Path, fdev: u64) -> io::Result<(PathBuf, PathBuf)> {
        let mut mount = file.to_path_buf();
        loop {
            let Some(parent) = mount.parent() else {
                // reached root '/'
                break;
            };
            let pmeta = fs::metadata(parent)?;
            if fdev != pmeta.dev() {
                // reached mount point
                break;
            }
            mount = parent.to_path_buf();
        }
        let rfile = file.strip_prefix(&mount).unwrap().to_path_buf();
        Ok((mount, rfile))
    }

    pub fn new(filename: &str) -> io::Result<EfiFile> {
        let file = fs::canonicalize(filename)?;
        let meta = fs::metadata(&file)?;
        let fdev = meta.dev();

        let (mount, rfile) = EfiFile::mount(&file, fdev)?;

        let device = udev::Device::from_devnum(udev::DeviceType::Block, fdev)?;

        Ok(EfiFile {
            file,
            fdev,
            mount,
            rfile,
            device,
        })
    }

    pub fn efi_name(&self) -> Option<NoCaseString> {
        let mut path = String::new();
        for elem in self.rfile.iter() {
            path.push('\\');
            path.push_str(elem.to_str()?);
        }
        Some(NoCaseString::new(&path))
    }

    pub fn basename(&self) -> Option<String> {
        let e = self.rfile.iter().last()?;
        let s = e.to_str()?;
        Some(s.to_string())
    }

    fn udev_prop(&self, name: &str) -> Option<String> {
        let os = self.device.property_value(name)?;
        let s = os.to_str()?;
        Some(s.to_string())
    }

    pub fn devpathnode_hd(&self) -> Option<devpath::DevPathNode> {
        let Some(nr_raw) = self.udev_prop("ID_PART_ENTRY_NUMBER") else {
            error!("missing ID_PART_ENTRY_NUMBER for 0x{:x}", self.fdev);
            return None;
        };
        let nr: u32 = nr_raw.parse().ok()?;
        let start: u64 = self.udev_prop("ID_PART_ENTRY_OFFSET")?.parse().ok()?;
        let size: u64 = self.udev_prop("ID_PART_ENTRY_SIZE")?.parse().ok()?;
        let guid_str = self.udev_prop("ID_PART_ENTRY_UUID")?;
        let guid = Guid::try_parse(&guid_str).ok()?;
        Some(devpath::DevPathNode::MediaHd(devpath::DevPathMediaHd {
            nr,
            start,
            size,
            signature: guid,
            mbrtype: 2, // GPT
            sigtype: 2, // UUID
        }))
    }

    pub fn devpathnode_file(&self) -> Option<devpath::DevPathNode> {
        let path = self.efi_name()?;
        Some(devpath::DevPathNode::MediaFilePath(path))
    }

    pub fn devpath(&self) -> Option<devpath::DevPath> {
        let mut devpath = devpath::DevPath::new();
        devpath.push(self.devpathnode_hd()?);
        devpath.push(self.devpathnode_file()?);
        Some(devpath)
    }
}
