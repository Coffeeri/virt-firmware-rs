#![cfg(target_arch = "x86_64")]

use virtfw_libhw::serial::{SerialWriter, SERIAL0_BASE};

pub static SERCON: SerialWriter = SerialWriter::new(SERIAL0_BASE);

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut linebuf = $crate::linebuf::LineBuf::new();
        let _ = core::write!(&mut linebuf, $($arg)*);
        $crate::sercon::SERCON.putsln(linebuf.as_bytes());
    }};
}

pub fn sercon_init() {
    SERCON.init();
    println!("serial console at 0x{:x}", SERCON.iobase);
}
