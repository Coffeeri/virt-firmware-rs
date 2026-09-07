//! efi data types

use core::cmp::Ordering;
use core::fmt;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[cfg(feature = "std")]
use chrono::{Datelike, Timelike, Utc};

/// EFI_TIME in edk2
#[repr(C)]
#[derive(
    Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Eq, PartialEq, Default,
)]
pub struct EfiTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    _pad1: u8,
    pub nanosecond: u32,
    pub timezone: i16,
    pub daylight: u8,
    _pad2: u8,
}

impl EfiTime {
    pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            ..Default::default()
        }
    }

    #[cfg(feature = "std")]
    pub fn now() -> Self {
        let t = Utc::now();
        Self {
            year: t.year() as u16,
            month: t.month() as u8,
            day: t.day() as u8,
            hour: t.hour() as u8,
            minute: t.minute() as u8,
            second: t.second() as u8,
            nanosecond: t.nanosecond(),
            ..Default::default()
        }
    }

    pub fn initial() -> Self {
        // should be old enough that it predates any secure boot
        // related authenticated variable time stamps.
        Self {
            year: 2010,
            month: 1,
            day: 1,
            ..Default::default()
        }
    }

    pub fn dbxupdate() -> Self {
        // used by microsoft for dbx updates
        Self {
            year: 2010,
            month: 3,
            day: 6,
            hour: 19,
            minute: 17,
            second: 21,
            ..Default::default()
        }
    }
}

#[cfg(feature = "x509")]
impl From<&der::DateTime> for EfiTime {
    fn from(dt: &der::DateTime) -> Self {
        Self {
            year: dt.year(),
            month: dt.month(),
            day: dt.day(),
            hour: dt.hour(),
            minute: dt.minutes(),
            second: dt.seconds(),
            ..Default::default()
        }
    }
}

impl PartialOrd for EfiTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EfiTime {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.year != other.year {
            return self.year.cmp(&other.year);
        }
        if self.month != other.month {
            return self.month.cmp(&other.month);
        }
        if self.day != other.day {
            return self.day.cmp(&other.day);
        }
        if self.hour != other.hour {
            return self.hour.cmp(&other.hour);
        }
        if self.minute != other.minute {
            return self.minute.cmp(&other.minute);
        }
        if self.second != other.second {
            return self.second.cmp(&other.second);
        }
        if self.nanosecond != other.nanosecond {
            return self.nanosecond.cmp(&other.nanosecond);
        }
        Ordering::Equal
    }
}

impl fmt::Display for EfiTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
