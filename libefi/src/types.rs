//! efi data types

use core::cmp::Ordering;
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
            nanosecond: 0,
            timezone: 0,
            daylight: 0,
            _pad1: 0,
            _pad2: 0,
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
