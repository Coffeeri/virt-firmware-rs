//! case insensitive string (for efi file names)
use std::fmt;

#[derive(Clone, Debug)]
pub struct NoCaseString(pub String);

impl NoCaseString {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl PartialEq for NoCaseString {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for NoCaseString {}

impl fmt::Display for NoCaseString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
