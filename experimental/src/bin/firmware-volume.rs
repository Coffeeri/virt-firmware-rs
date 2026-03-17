use clap::{CommandFactory, Parser};
use core::marker::PhantomData;
use core::mem::{offset_of, size_of};
use core::ops::Range;
use log::{error, info};
use std::process::ExitCode;
use uguid::Guid;
use zerocopy::{FromBytes, KnownLayout};

use virtfw_libefi::guids::{pretty_name, Ffs, LzmaCompress, RawGuid};

// ---------------------------------------------------------------------
// helper trait for parsing firmware images

pub trait FirmwareItem: FromBytes {
    const ALIGN: usize;
    fn datarange(&self) -> Range<usize>;
    fn try_type_name(&self) -> Option<&str>;

    fn new<'b>(blob: &'b [u8]) -> Result<Self, Box<dyn std::error::Error + 'b>>
    where
        Self: Sized,
        Self: KnownLayout,
        Self: 'b,
    {
        let (hdr, _) = Self::read_from_prefix(blob)?;
        if !hdr.is_valid() {
            return Err("header is not valid".into());
        }
        Ok(hdr)
    }

    fn type_name(&self) -> &str {
        self.try_type_name().unwrap_or("<unknown>")
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn size(&self) -> usize {
        self.datarange().end
    }

    fn pad_size(&self) -> usize {
        (self.size() + Self::ALIGN - 1) & !(Self::ALIGN - 1)
    }
}

pub struct FirmwareIter<'b, T> {
    offset: usize,
    blob: &'b [u8],
    phantom: PhantomData<T>,
}

impl<'b, T> FirmwareIter<'b, T> {
    pub fn new(blob: &'b [u8]) -> FirmwareIter<'b, T> {
        Self {
            offset: 0,
            blob,
            phantom: PhantomData,
        }
    }
}

impl<'b, T> Iterator for FirmwareIter<'b, T>
where
    T: FirmwareItem,
    T: KnownLayout,
{
    type Item = (T, &'b [u8]);

    fn next(&mut self) -> Option<(T, &'b [u8])> {
        if self.offset + size_of::<T>() > self.blob.len() {
            return None;
        }
        let blob = &self.blob[self.offset..];
        let hdr = T::new(blob).ok()?;
        let data = blob.get(..hdr.size())?;
        self.offset += hdr.pad_size();
        Some((hdr, data))
    }
}

// ---------------------------------------------------------------------
// EFI_COMMON_SECTION_HEADER

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C)]
pub struct FileSectionHeader {
    size: [u8; 3],
    stype: u8,
    // only in EFI_COMMON_SECTION_HEADER2
    ext_size: u32,
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C)]
pub struct FileSectionGuidDefined {
    guid: RawGuid,
    data_offset: u16,
    attributes: u16,
}

impl FirmwareItem for FileSectionHeader {
    const ALIGN: usize = 4;

    fn datarange(&self) -> Range<usize> {
        let is_large = self.size[0] == 0xff && self.size[1] == 0xff && self.size[2] == 0xff;
        if is_large {
            size_of::<Self>()..self.ext_size as usize
        } else {
            let start = size_of::<Self>() - size_of::<u32>();
            let end = self.size[0] as usize
                | (self.size[1] as usize) << 8
                | (self.size[2] as usize) << 16;
            start..end
        }
    }

    fn try_type_name(&self) -> Option<&str> {
        match self.stype {
            0x01 => Some("compressed"),
            0x02 => Some("guid-defined"),
            0x10 => Some("pe32"),
            0x11 => Some("pic"),
            0x12 => Some("te"),
            0x13 => Some("dxe-depex"),
            0x14 => Some("version"),
            0x15 => Some("ui"),
            0x16 => Some("compat16"),
            0x17 => Some("fw-volume"),
            0x18 => Some("freeform-guid"),
            0x19 => Some("raw"),
            0x1b => Some("pei-depex"),
            0x1c => Some("smm-depex"),
            _ => None,
        }
    }
}

impl core::fmt::Display for FileSectionHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "length={} type=0x{:x} [{}]",
            self.size(),
            self.stype,
            self.type_name()
        )
    }
}

// ---------------------------------------------------------------------
// EFI_FFS_FILE_HEADER

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C)]
pub struct FfsFileHeader {
    name: RawGuid,
    check: u16,
    ftype: u8,
    attr: u8,
    size: [u8; 3],
    state: u8,
    // only in EFI_FFS_FILE_HEADER2
    ext_size: u64,
}

impl FfsFileHeader {
    const FFS_ATTRIB_LARGE_FILE: u8 = 0x01;

    pub fn has_sections(&self) -> bool {
        matches!(
            self.ftype,
            0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x08 | 0x09 | 0x0a | 0x0b | 0x0d | 0x0e | 0x0f
        )
    }
}

impl FirmwareItem for FfsFileHeader {
    const ALIGN: usize = 8;

    fn datarange(&self) -> Range<usize> {
        let is_large = (self.attr & Self::FFS_ATTRIB_LARGE_FILE) != 0;
        if is_large {
            size_of::<Self>()..self.ext_size as usize
        } else {
            let start = size_of::<Self>() - size_of::<u64>();
            let end = self.size[0] as usize
                | (self.size[1] as usize) << 8
                | (self.size[2] as usize) << 16;
            start..end
        }
    }

    fn try_type_name(&self) -> Option<&str> {
        match self.ftype {
            0x01 => Some("raw"),
            0x02 => Some("freeform"),
            0x03 => Some("sec-core"),
            0x04 => Some("pei-core"),
            0x05 => Some("dxe-core"),
            0x06 => Some("peim"),
            0x07 => Some("driver"),
            0x09 => Some("application"),
            0x0a => Some("mm"),
            0x0b => Some("fw-volume"),
            0x0d => Some("mm-core"),
            0x0e => Some("mm-standalone"),
            0x0f => Some("mm-standalone-core"),
            0xf0 => Some("padding"),
            _ => None,
        }
    }
}

impl core::fmt::Display for FfsFileHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let name: Guid = self.name.into();
        write!(
            f,
            "name={} length={} type=0x{:x} [{}]",
            pretty_name(&name),
            self.size(),
            self.ftype,
            self.type_name(),
        )
    }
}

// ---------------------------------------------------------------------
// EFI_FIRMWARE_VOLUME_HEADER

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C)]
pub struct FirmwareVolumeHeader {
    zero_vector: [u8; 16],
    fs_guid: RawGuid,
    fv_length: u64,
    signature: [u8; 4],
    attributes: u32, // FIXME
    hdr_length: u16,
    checksum: u16,
    ext_hdr_offset: u16,
    _reserved: u8,
    revision: u8,
    // EFI_FV_BLOCK_MAP_ENTRY array
    // EFI_FIRMWARE_VOLUME_EXT_HEADER
    ext: [u8; 64],
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C)]
pub struct FirmwareVolumeBlockMap {
    num_blocks: u32,
    length: u32,
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C)]
pub struct FirmwareVolumeExtHeader {
    fv_name: RawGuid,
    ext_hdr_size: u32,
}

impl FirmwareVolumeHeader {
    const SIGNATURE: [u8; 4] = *b"_FVH";

    pub fn fs_guid(&self) -> Guid {
        self.fs_guid.into()
    }

    pub fn is_ffs(&self) -> bool {
        self.fs_guid() == Ffs
    }

    pub fn name_guid(&self) -> Option<Guid> {
        if self.ext_hdr_offset == 0 {
            return None;
        }
        let off = self.ext_hdr_offset as usize - offset_of!(Self, ext);
        let blob = self.ext.get(off..)?;
        let (ehdr, _) = FirmwareVolumeExtHeader::read_from_prefix(blob).ok()?;
        Some(ehdr.fv_name.into())
    }
}

impl FirmwareItem for FirmwareVolumeHeader {
    const ALIGN: usize = 8;

    fn datarange(&self) -> Range<usize> {
        self.hdr_length as usize..self.fv_length as usize
    }

    fn try_type_name(&self) -> Option<&str> {
        None
    }

    fn is_valid(&self) -> bool {
        if self.signature != Self::SIGNATURE {
            return false;
        }
        true
    }
}

impl core::fmt::Display for FirmwareVolumeHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "guid={} length={}",
            pretty_name(&self.fs_guid()),
            self.size(),
        )?;
        if let Some(guid) = self.name_guid() {
            write!(f, " name={}", pretty_name(&guid))?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------
// helper structs to recursively walk firmware data structures
// the knowledge which element contains which sub-elements etc is
// in FirmwareItemContent::walk()

pub enum FirmwareItemHeader {
    Image,
    Volume(FirmwareVolumeHeader),
    File(FfsFileHeader),
    Section(FileSectionHeader),
}

pub struct FirmwareItemContent<'b> {
    hdr: FirmwareItemHeader,
    blob: &'b [u8],
}

impl<'b> core::fmt::Display for FirmwareItemContent<'b> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self.hdr {
            FirmwareItemHeader::Image => write!(fmt, "image: size={}", self.blob.len()),
            FirmwareItemHeader::Volume(v) => write!(fmt, "volume: {v}"),
            FirmwareItemHeader::File(f) => write!(fmt, "file: {f}"),
            FirmwareItemHeader::Section(s) => write!(fmt, "section: {s}"),
        }
    }
}

type FirmwareItemWalkFunc = fn(level: usize, item: &FirmwareItemContent);

impl<'b> FirmwareItemContent<'b> {
    pub fn new_image(blob: &'b [u8]) -> Self {
        Self {
            hdr: FirmwareItemHeader::Image,
            blob,
        }
    }

    pub fn new_volume(hdr: FirmwareVolumeHeader, blob: &'b [u8]) -> Self {
        Self {
            hdr: FirmwareItemHeader::Volume(hdr),
            blob,
        }
    }

    pub fn new_file(hdr: FfsFileHeader, blob: &'b [u8]) -> Self {
        Self {
            hdr: FirmwareItemHeader::File(hdr),
            blob,
        }
    }

    pub fn new_section(hdr: FileSectionHeader, blob: &'b [u8]) -> Self {
        Self {
            hdr: FirmwareItemHeader::Section(hdr),
            blob,
        }
    }

    pub fn walk_volumes(&self, level: usize, func: FirmwareItemWalkFunc, data: &'b [u8]) {
        FirmwareIter::<FirmwareVolumeHeader>::new(data)
            .map(|(h, b)| Self::new_volume(h, b))
            .for_each(|v| v.walk(level + 1, func));
    }

    pub fn walk_sections(&self, level: usize, func: FirmwareItemWalkFunc, data: &'b [u8]) {
        FirmwareIter::<FileSectionHeader>::new(data)
            .map(|(h, b)| Self::new_section(h, b))
            .for_each(|v| v.walk(level + 1, func));
    }

    pub fn walk(&self, level: usize, func: FirmwareItemWalkFunc) {
        func(level, self);
        match &self.hdr {
            FirmwareItemHeader::Image => {
                self.walk_volumes(level, func, self.blob);
            }
            FirmwareItemHeader::Volume(v) => {
                let data = self.blob.get(v.datarange()).unwrap();
                if v.is_ffs() {
                    FirmwareIter::<FfsFileHeader>::new(data)
                        .map(|(h, b)| Self::new_file(h, b))
                        .for_each(|v| v.walk(level + 1, func));
                }
            }
            FirmwareItemHeader::File(f) => {
                let data = self.blob.get(f.datarange()).unwrap();
                if f.has_sections() {
                    self.walk_sections(level, func, data);
                }
            }
            FirmwareItemHeader::Section(s) => {
                let data = self.blob.get(s.datarange()).unwrap();
                match s.stype {
                    0x02 => {
                        if let Ok((hdr, _)) = FileSectionGuidDefined::read_from_prefix(data) {
                            let guid: Guid = hdr.guid.into();
                            if guid == LzmaCompress {
                                let xz = self.blob.get(hdr.data_offset as usize..).unwrap();
                                let unxz = lzma::decompress(xz).expect("uncompress");
                                self.walk_sections(level, func, &unxz);
                            }
                        }
                    }
                    0x17 => {
                        self.walk_volumes(level, func, data);
                    }
                    _ => {}
                }
            }
        }
    }
}

// ---------------------------------------------------------------------
// test app

#[derive(Parser, Debug)]
#[command(version, author, name = "firmware-volume",
          about = "parse edk2 firmware images",
          long_about = None)]
struct Args {
    /// files to inspect
    #[arg()]
    files: Vec<String>,

    #[arg(long, hide = true)]
    manpage: bool,
}

fn walk_print(level: usize, item: &FirmwareItemContent) {
    println!("{:indent$}{item}", "", indent = level * 2)
}

fn parse_file(filename: &str) {
    info!("reading {filename}");
    let res = std::fs::read(filename);
    if let Err(e) = res {
        error!("read: {e}");
        return;
    }

    let blob = res.unwrap();
    FirmwareItemContent::new_image(&blob).walk(1, walk_print);
}

fn main() -> ExitCode {
    let cfg = Args::parse();

    if cfg.manpage {
        let man = clap_mangen::Man::new(Args::command());
        man.render(&mut std::io::stdout()).expect("render manpage");
        return 0.into();
    }

    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Debug)
        .init()
        .unwrap();

    for arg in &cfg.files {
        parse_file(arg);
    }

    0.into()
}
