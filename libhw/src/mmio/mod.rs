mod arm;
mod x86;

#[cfg(target_arch = "aarch64")]
pub use crate::mmio::arm::{readb, readl, readq, readw, writeb, writel, writeq, writew};
#[cfg(target_arch = "x86_64")]
pub use crate::mmio::x86::{readb, readl, readq, readw, writeb, writel, writeq, writew};
