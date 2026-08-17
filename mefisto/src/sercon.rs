#![cfg(target_arch = "x86_64")]

use log::info;

use virtfw_libhw::serial::{SerialWriter, SERIAL0_BASE};

pub static SERCON: SerialWriter = SerialWriter::new_io(SERIAL0_BASE);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut linebuf = $crate::linebuf::LineBuf::new();
        let _ = core::write!(&mut linebuf, $($arg)*);
        $crate::sercon::SERCON.puts(linebuf.as_bytes());
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut linebuf = $crate::linebuf::LineBuf::new();
        let _ = core::write!(&mut linebuf, $($arg)*);
        $crate::sercon::SERCON.putsln(linebuf.as_bytes());
    }};
}

pub fn sercon_init(loglevel: log::LevelFilter) {
    SERCON.init();
    log::set_logger(&SERCON).unwrap();
    log::set_max_level(loglevel);
    info!("serial console at {}", SERCON);
}
