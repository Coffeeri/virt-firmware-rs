use virtfw_libhw::size::PAGE_4K;

#[repr(C, align(4096))]
pub struct SecretsPage {
    pub vmpl0_seq_lo: u32,
    pub vmpl1_seq_lo: u32,
    pub vmpl2_seq_lo: u32,
    pub vmpl3_seq_lo: u32,
    pub ap_jump_table_pa: u64,
    pub vmpl0_seq_hi: u32,
    pub vmpl1_seq_hi: u32,
    pub vmpl2_seq_hi: u32,
    pub vmpl3_seq_hi: u32,
    _res5: [u8; 0x16],
    pub version: u16,
    pub guest: [u8; PAGE_4K - 0x40],
}

#[link_section = ".snp.secrets"]
pub static SECRETS: SecretsPage = unsafe { core::mem::zeroed() };
