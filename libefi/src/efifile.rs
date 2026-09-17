extern crate std;
use log::{debug, error};
use std::format;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
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
    fn mount(file: &Path, fdev: u64) -> io::Result<PathBuf> {
        let mut mount = file.to_path_buf();
        while let Some(parent) = mount.parent() {
            let pmeta = fs::metadata(parent)?;
            if fdev != pmeta.dev() {
                // reached mount point
                break;
            }
            mount = parent.to_path_buf();
        }
        debug!("mount point for {} is {}", file.display(), mount.display());
        Ok(mount)
    }

    pub fn new(filename: &str) -> io::Result<EfiFile> {
        let file = match fs::canonicalize(filename) {
            Ok(f) => f,
            Err(_) => PathBuf::from(filename),
        };
        let meta = match fs::metadata(&file) {
            Ok(m) => m,
            Err(_) => {
                // file might not exist (due to kernel uninstall),
                // try use parent directory instead then
                let dir = file.parent().expect("get parent dir");
                fs::metadata(dir)?
            }
        };
        let fdev = meta.dev();

        let mount = EfiFile::mount(&file, fdev)?;
        let rfile = file.strip_prefix(&mount).unwrap().to_path_buf();

        let device = udev::Device::from_devnum(udev::DeviceType::Block, fdev)?;

        Ok(EfiFile {
            file,
            fdev,
            mount,
            rfile,
            device,
        })
    }

    fn device_from_partuuid(guid: &Guid) -> Option<u64> {
        let mut path = PathBuf::from("/dev/disk/by-partuuid");
        path.push(guid.to_string());
        let meta = fs::metadata(&path).ok()?;
        Some(meta.rdev())
    }

    fn mount_from_device(device: u64) -> Option<PathBuf> {
        let major = libc::major(device);
        let minor = libc::minor(device);
        let search = format!("{major}:{minor}");

        let mountinfo = fs::read_to_string("/proc/self/mountinfo").ok()?;
        for line in mountinfo.lines() {
            let mut iter = line.split_whitespace();
            let _ = iter.next();
            let _ = iter.next();
            let device = iter.next()?;
            let _ = iter.next();
            let mount = iter.next()?;
            if search == device {
                return Some(mount.into());
            }
        }
        None
    }

    fn rfile_from_efi_name(efi_name: &NoCaseString) -> PathBuf {
        let mut rfile = PathBuf::new();
        for item in efi_name.0.split("\\") {
            rfile.push(item);
        }
        rfile
    }

    pub fn new_from_dp(dp: &devpath::DevPath) -> Option<Self> {
        // MediaHd -> mount point
        let hd_node = dp.nodes.first()?;
        let devpath::DevPathNode::MediaHd(part) = hd_node else {
            return None;
        };

        let rdev = Self::device_from_partuuid(&part.signature)?;
        let mount = Self::mount_from_device(rdev)?;

        // MediaFilePath -> relative PathBuf
        let file_node = dp.nodes.get(1)?;
        let devpath::DevPathNode::MediaFilePath(filename) = file_node else {
            return None;
        };
        let rfile = Self::rfile_from_efi_name(filename);

        let file = mount.join(&rfile);
        let device = udev::Device::from_devnum(udev::DeviceType::Block, rdev).ok()?;
        Some(EfiFile {
            file,
            fdev: rdev,
            mount,
            rfile,
            device,
        })
    }

    pub fn new_with_efi_name(self, efi_name: &NoCaseString) -> Self {
        let rfile = Self::rfile_from_efi_name(efi_name);
        let file = self.mount.join(&rfile);
        EfiFile {
            file,
            fdev: self.fdev,
            mount: self.mount,
            rfile,
            device: self.device,
        }
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
        let e = self.rfile.iter().next_back()?;
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
