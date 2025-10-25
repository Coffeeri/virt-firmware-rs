use core::ops::Range;

pub const KIB: usize = 1 << 10;
pub const MIB: usize = 1 << 20;
pub const GIB: usize = 1 << 30;

pub fn pretty_usize(size: usize) -> (usize, &'static str) {
    if size > 10 * GIB {
        (size / GIB, " GB")
    } else if size > 10 * MIB {
        (size / MIB, " MB")
    } else if size > 10 * KIB {
        (size / KIB, " kB")
    } else if size > 1 {
        (size, " bytes")
    } else {
        (size, " byte")
    }
}

pub fn pretty_range(range: &Range<usize>) -> (usize, &'static str) {
    let size = range.end - range.start;
    pretty_usize(size)
}
