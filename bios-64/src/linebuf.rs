use core::fmt::{Error, Result, Write};

const BUFSIZE: usize = 80;

pub struct LineBuf {
    buf: [u8; BUFSIZE],
    len: usize,
}

impl LineBuf {
    pub const fn new() -> Self {
        Self {
            buf: [0; BUFSIZE],
            len: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[0..self.len]
    }
}

impl Default for LineBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for LineBuf {
    fn write_str(&mut self, s: &str) -> Result {
        let sbytes = s.as_bytes();
        if self.len + sbytes.len() > BUFSIZE {
            // signal an error on overflow
            return Err(Error);
        }
        self.buf[self.len..self.len + sbytes.len()].copy_from_slice(sbytes);
        self.len += sbytes.len();
        Ok(())
    }
}
