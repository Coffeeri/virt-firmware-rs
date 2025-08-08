use log::debug;
use std::fmt;
use std::mem::size_of;
use uguid::{guid, Guid};
use zerocopy::byteorder::little_endian::U16;
use zerocopy::{FromBytes, Unalign};

pub const OVMF_META_LIST: Guid = guid!("96b582de-1fb2-45f7-baea-a366c55a082d");
pub const OVMF_META_SEV_RESET: Guid = guid!("00f771de-1a7e-4fcb-890e-68c77e2fb44e");
pub const OVMF_META_SEV_SECRET: Guid = guid!("4c2eb361-7d9b-4cc3-8081-127c90d3d294");
pub const OVMF_META_SEV_HASHES: Guid = guid!("7255371f-3a3b-4b04-927b-1da6efa8d454");
pub const OVMF_META_SEV_LIST: Guid = guid!("dc886566-984a-4798-a75e-5585a7bf67cc");
pub const OVMF_META_TDX_LIST: Guid = guid!("e47a6535-984a-4798-865e-4685a7bf8ec2");
pub const OVMF_META_IGVM_PARAM: Guid = guid!("784fa70e-3176-4677-8a20-04b68699e374");

#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct OvmfMetaBlock {
    size: Unalign<U16>,
    guid: [u8; 16],
}

#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct OvmfMetaListHead {
    magic: u32,
    length: u32,
    version: u32,
    entries: u32,
}

#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct OvmfMetaSevIgvmEntry {
    base: u32,
    size: u32,
    kind: u32,
}

#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct OvmfMetaTdxEntry {
    file_base: u32,
    file_size: u32,
    mem_base: u64,
    mem_size: u64,
    kind: u32,
    flags: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OvmfRegionType {
    Undefined,

    SevMemory,
    SevSecrets,
    SevCpuid,
    SevSvsmCca,
    SevHashes,

    TdxFvCode,
    TdxFvVars,
    TdxTdHob,
    TdxMemory,

    IgvmParamArea,
    IgvmParamMemoryMap,
    IgvmHobArea,
}

impl OvmfRegionType {
    pub fn new_sev(t: u32) -> OvmfRegionType {
        match t {
            0x01 => OvmfRegionType::SevMemory,
            0x02 => OvmfRegionType::SevSecrets,
            0x03 => OvmfRegionType::SevCpuid,
            0x04 => OvmfRegionType::SevSvsmCca,
            0x10 => OvmfRegionType::SevHashes,
            _ => OvmfRegionType::Undefined,
        }
    }

    pub fn new_igvm(t: u32) -> OvmfRegionType {
        match t {
            0x100 => OvmfRegionType::IgvmParamArea,
            0x101 => OvmfRegionType::IgvmParamMemoryMap,
            0x200 => OvmfRegionType::IgvmHobArea,
            _ => OvmfRegionType::Undefined,
        }
    }

    pub fn new_tdx(t: u32) -> OvmfRegionType {
        match t {
            0x00 => OvmfRegionType::TdxFvCode,
            0x01 => OvmfRegionType::TdxFvVars,
            0x02 => OvmfRegionType::TdxTdHob,
            0x03 => OvmfRegionType::TdxMemory,
            _ => OvmfRegionType::Undefined,
        }
    }
}

#[derive(Debug)]
pub struct OvmfRegion {
    pub memory: (usize, usize),
    pub file: Option<(u32, u32)>,
    pub etype: OvmfRegionType,
}

impl OvmfRegion {
    fn new_sev(entry: &OvmfMetaSevIgvmEntry) -> OvmfRegion {
        OvmfRegion {
            memory: (entry.base as usize, entry.size as usize),
            etype: OvmfRegionType::new_sev(entry.kind),
            ..Default::default()
        }
    }

    fn new_igvm(entry: &OvmfMetaSevIgvmEntry) -> OvmfRegion {
        OvmfRegion {
            memory: (entry.base as usize, entry.size as usize),
            etype: OvmfRegionType::new_igvm(entry.kind),
            ..Default::default()
        }
    }

    fn new_tdx(entry: &OvmfMetaTdxEntry) -> OvmfRegion {
        OvmfRegion {
            memory: (entry.mem_base as usize, entry.mem_size as usize),
            file: if entry.file_size != 0 {
                Some((entry.file_base, entry.file_size))
            } else {
                None
            },
            etype: OvmfRegionType::new_tdx(entry.kind),
        }
    }
}

impl Default for OvmfRegion {
    fn default() -> OvmfRegion {
        OvmfRegion {
            memory: (0, 0),
            file: None,
            etype: OvmfRegionType::Undefined,
        }
    }
}

impl fmt::Display for OvmfRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "0x{:08x} +0x{:x}  {:?}",
            self.memory.0, self.memory.1, self.etype
        )?;
        if let Some(r) = self.file {
            write!(f, "  (0x{:x} +0x{:x})", r.0, r.1)?;
        };
        Ok(())
    }
}

pub struct OvmfMeta<'f> {
    firmware: &'f [u8],
    pub regions: Vec<OvmfRegion>,
}

impl OvmfMeta<'_> {
    fn meta_blk(blob: &[u8]) -> Option<(Guid, &[u8], &[u8])> {
        let (data, meta) = OvmfMetaBlock::read_from_suffix(blob).ok()?;
        let size: u16 = meta.size.get().into();
        let dsize = (size as usize).checked_sub(size_of::<OvmfMetaBlock>())?;
        let start = data.len().checked_sub(dsize)?;
        let (other, blk) = data.split_at_checked(start)?;
        Some((Guid::from_bytes(meta.guid), blk, other))
    }

    fn parse_blk(&mut self, guid: Guid, blk: &[u8]) -> Option<()> {
        debug!("item: guid {}, size {}", guid, blk.len());
        match guid {
            OVMF_META_SEV_RESET => {
                let (addr, _) = u32::read_from_prefix(blk).ok()?;
                debug!("  sev reset: 0x{addr:x}");
            }
            OVMF_META_SEV_SECRET => {
                let (addr, r) = u32::read_from_prefix(blk).ok()?;
                let (size, _) = u32::read_from_prefix(r).ok()?;
                debug!("  sev secret 0x{addr:x} +0x{size:x}");
            }
            OVMF_META_SEV_HASHES => {
                let (addr, r) = u32::read_from_prefix(blk).ok()?;
                let (size, _) = u32::read_from_prefix(r).ok()?;
                debug!("  sev hashes 0x{addr:x} +0x{size:x}",);
            }
            OVMF_META_SEV_LIST => {
                let (listbase, _) = u32::read_from_prefix(blk).ok()?;
                let start = self.firmware.len().checked_sub(listbase as usize)?;
                debug!("  sev addr 0x{listbase:x} -> 0x{start:x}");
                let (_, sev) = self.firmware.split_at_checked(start)?;
                let (head, mut items) = OvmfMetaListHead::read_from_prefix(sev).ok()?;
                debug!("    {head:?}");
                for _ in 0..head.entries {
                    let (e, r) = OvmfMetaSevIgvmEntry::read_from_prefix(items).ok()?;
                    if e.size != 0 {
                        let region = OvmfRegion::new_sev(&e);
                        self.regions.push(region);
                    }
                    items = r;
                }
            }
            OVMF_META_TDX_LIST => {
                let (listbase, _) = u32::read_from_prefix(blk).ok()?;
                let start = self.firmware.len().checked_sub(listbase as usize)?;
                debug!("  tdx addr 0x{listbase:x} -> 0x{start:x}");
                let (_, tdx) = self.firmware.split_at_checked(start)?;
                let (head, mut items) = OvmfMetaListHead::read_from_prefix(tdx).ok()?;
                debug!("    {head:?}");
                for _ in 0..head.entries {
                    let (e, r) = OvmfMetaTdxEntry::read_from_prefix(items).ok()?;
                    if e.mem_size != 0 {
                        let region = OvmfRegion::new_tdx(&e);
                        self.regions.push(region);
                    }
                    items = r;
                }
            }
            OVMF_META_IGVM_PARAM => {
                let (listbase, _) = u32::read_from_prefix(blk).ok()?;
                let start = self.firmware.len().checked_sub(listbase as usize)?;
                debug!("  igvm param 0x{listbase:x} -> 0x{start:x}");
                let (_, param) = self.firmware.split_at_checked(start)?;
                let (head, mut items) = OvmfMetaListHead::read_from_prefix(param).ok()?;
                debug!("    {head:?}");
                for _ in 0..head.entries {
                    let (e, r) = OvmfMetaSevIgvmEntry::read_from_prefix(items).ok()?;
                    if e.size != 0 {
                        debug!("    {e:?}");
                        let region = OvmfRegion::new_igvm(&e);
                        self.regions.push(region);
                    }
                    items = r;
                }
            }
            _ => {
                debug!("  unknown");
            }
        }
        Some(())
    }

    pub fn new(firmware: &[u8]) -> Option<OvmfMeta<'_>> {
        let mut ovmfmeta = OvmfMeta {
            firmware,
            regions: Vec::new(),
        };

        let s = firmware.len().checked_sub(0x20)?;
        let (f, _) = firmware.split_at(s);
        let (g, mut m1, _) = Self::meta_blk(f)?;

        if g == OVMF_META_LIST {
            loop {
                let Some((g, b, m2)) = Self::meta_blk(m1) else {
                    break;
                };
                ovmfmeta.parse_blk(g, b);
                m1 = m2;
            }
        }

        Some(ovmfmeta)
    }

    pub fn print(&self) {
        println!("OVMF regions");
        for r in &self.regions {
            println!("  {r}");
        }
    }
}
