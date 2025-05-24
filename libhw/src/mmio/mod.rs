mod arm;
mod x86;

#[cfg(target_arch = "aarch64")]
pub use crate::mmio::arm::{readb, readl, readw, writeb, writel, writew};
#[cfg(target_arch = "x86_64")]
pub use crate::mmio::x86::{readb, readl, readw, writeb, writel, writew};
