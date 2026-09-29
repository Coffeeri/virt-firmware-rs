use uefi::prelude::*;
use uefi::print;
use uefi::proto::console::text::{Input, Key};

pub struct Pager {
    enabled: bool,
    index: u32,
    count: u32,
}

impl Pager {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            index: 0,
            count: 20,
        }
    }

    fn wait_read_key(stdin: &mut Input) -> Option<Key> {
        let key_event = stdin.wait_for_key_event().unwrap();
        // mut is needed by uefi < 0.41
        #[allow(clippy::unnecessary_mut_passed)]
        boot::wait_for_event(&mut [key_event]).unwrap();
        stdin.read_key().unwrap()
    }

    fn wait_stdin_key() {
        let _key = system::with_stdin(Self::wait_read_key);
    }

    pub fn lines(&mut self, n: u32) {
        if !self.enabled {
            return;
        }

        self.index += n;
        if self.index < self.count {
            return;
        }

        print!("--- more ---\r");
        Self::wait_stdin_key();
        print!("            \r");
        self.index = 0;
    }

    pub fn end(&mut self) {
        if self.index == 0 {
            return;
        }

        print!("--- end ---\r");
        Self::wait_stdin_key();
        print!("           \r");
        self.index = 0;
    }
}
