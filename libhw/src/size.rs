use core::ops::Range;

pub const KIB: usize = 1 << 10;
pub const MIB: usize = 1 << 20;
pub const GIB: usize = 1 << 30;
pub const TIB: usize = 1 << 40;

pub const PAGE_4K: usize = 4 * KIB;
pub const PAGE_2M: usize = 2 * MIB;
pub const PAGE_1G: usize = GIB;

pub fn pretty_usize(size: usize) -> (usize, &'static str) {
    if size > 10 * TIB {
        (size / TIB, " TB")
    } else if size > 10 * GIB {
        (size / GIB, " GB")
    } else if size > 10 * MIB {
        (size / MIB, " MB")
    } else if size > 10 * KIB {
        (size / KIB, " kB")
    } else if size == 1 {
        (size, " byte")
    } else {
        (size, " bytes")
    }
}

pub fn pretty_range(range: &Range<usize>) -> (usize, &'static str) {
    let size = range.end - range.start;
    pretty_usize(size)
}
