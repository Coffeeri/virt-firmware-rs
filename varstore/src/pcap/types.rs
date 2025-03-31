use zerocopy_derive::FromBytes;

pub const PCAP_MAGIC: u32 = 0xa1b2c3d4;

#[derive(Debug, FromBytes)]
#[repr(C)]
pub struct PcapFileHeader {
    pub magic: u32,
    pub major: u16,
    pub minor: u16,
    pub tz: i32,
    pub flags: u32,
    pub snaplen: u32,
    pub network: u32,
}

#[derive(Debug, FromBytes)]
#[repr(C)]
pub struct PcapPacketHeader {
    pub ts_sec: u32,
    pub ts_usec: u32,
    pub incl_len: u32,
    pub orig_len: u32,
}
