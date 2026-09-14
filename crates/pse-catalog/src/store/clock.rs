// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The one place the platform reads the wall clock (blueprint §20.2 `created_at`).
//!
//! Wall-clock time is an ambient input, and §20.4's reproduction asserts that
//! deterministic passes reproduce their logical hashes. A timestamp reached through a
//! global function is unmockable and quietly makes every manifest unique; a timestamp
//! reached through [`Clock`] is a constructor argument a test can pin. `created_at` is
//! excluded from logical membership (§20.2) precisely so that this seam costs nothing.
//!
//! The formatting is hand-rolled from [`SystemTime`] rather than taken from `chrono`,
//! which is not a dependency of this workspace and would be a whole date-time family to
//! resolve for one line of output.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Days in each month of a non-leap year.
const MONTH_LENGTHS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Seconds in a day.
const SECONDS_PER_DAY: u64 = 86_400;

/// A source of the current time, in RFC 3339 UTC (blueprint §20.2).
pub trait Clock: Send + Sync + fmt::Debug {
    /// The current instant, as `YYYY-MM-DDTHH:MM:SSZ`.
    fn now_rfc3339_utc(&self) -> String;
}

/// The deployment clock: the host's wall clock, rendered in UTC.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_rfc3339_utc(&self) -> String {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        format_epoch_seconds(seconds)
    }
}

/// A clock that always reports the same instant, for fixtures and golden manifests.
///
/// Deliberately unvalidated: a test that wants to see what an out-of-range timestamp does
/// to a manifest must be able to write one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedClock(pub String);

impl Clock for FixedClock {
    fn now_rfc3339_utc(&self) -> String {
        self.0.clone()
    }
}

/// Renders seconds since the Unix epoch as `YYYY-MM-DDTHH:MM:SSZ`.
///
/// Whole seconds: `created_at` is an audit record, not an ordering key, and a fractional
/// part would be one more thing two implementations could disagree about.
fn format_epoch_seconds(seconds: u64) -> String {
    let days = seconds / SECONDS_PER_DAY;
    let time_of_day = seconds % SECONDS_PER_DAY;
    let (year, month, day) = civil_from_days(days);
    let hour = time_of_day / 3_600;
    let minute = (time_of_day % 3_600) / 60;
    let second = time_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
fn is_leap_year(year: u64) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

/// Calendar-valid UTC form shared by physical manifest/ref admission: four-digit
/// positive year, seconds 00–59, and optionally one through nine fractional digits.
pub(crate) fn valid_rfc3339_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(20..=30).contains(&bytes.len())
        || bytes.last() != Some(&b'Z')
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return false;
    }
    if bytes.len() != 20
        && (bytes.len() < 22
            || bytes[19] != b'.'
            || !bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit))
    {
        return false;
    }
    let number = |range: std::ops::Range<usize>| {
        bytes[range].iter().try_fold(0u64, |value, byte| {
            byte.is_ascii_digit()
                .then(|| value * 10 + u64::from(byte - b'0'))
        })
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0..4),
        number(5..7),
        number(8..10),
        number(11..13),
        number(14..16),
        number(17..19),
    ) else {
        return false;
    };
    if year == 0
        || !(1..=12).contains(&month)
        || day == 0
        || hour > 23
        || minute > 59
        || second > 59
    {
        return false;
    }
    let Some(month) = usize::try_from(month - 1).ok() else {
        return false;
    };
    let days = u64::from(MONTH_LENGTHS[month]) + u64::from(month == 1 && is_leap_year(year));
    day <= days
}

/// The number of days in `year`.
fn days_in_year(year: u64) -> u64 {
    if is_leap_year(year) { 366 } else { 365 }
}

/// The civil date `days` after 1970-01-01.
///
/// A plain forward walk rather than Howard Hinnant's closed form: this runs once per
/// publication, and a loop a reader can check by hand is worth more here than four lines
/// of modular arithmetic nobody will re-derive.
fn civil_from_days(days: u64) -> (u64, u32, u32) {
    let mut year = 1970_u64;
    let mut remaining = days;
    loop {
        let length = days_in_year(year);
        if remaining < length {
            break;
        }
        remaining -= length;
        year += 1;
    }

    let mut month = 0_usize;
    while month < MONTH_LENGTHS.len() {
        let mut length = u64::from(MONTH_LENGTHS[month]);
        if month == 1 && is_leap_year(year) {
            length += 1;
        }
        if remaining < length {
            break;
        }
        remaining -= length;
        month += 1;
    }

    let month_number = u32::try_from(month + 1).unwrap_or(1);
    let day_number = u32::try_from(remaining + 1).unwrap_or(1);
    (year, month_number, day_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_renders_as_the_epoch() {
        assert_eq!(format_epoch_seconds(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn known_instants_render_correctly() {
        // Values cross-checked against `date -u -d @<seconds>`.
        let cases = [
            (1_u64, "1970-01-01T00:00:01Z"),
            (86_399, "1970-01-01T23:59:59Z"),
            (86_400, "1970-01-02T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_000_000_000, "2001-09-09T01:46:40Z"),
            (1_709_164_800, "2024-02-29T00:00:00Z"),
            (1_735_689_599, "2024-12-31T23:59:59Z"),
            (1_767_225_600, "2026-01-01T00:00:00Z"),
        ];
        for (seconds, expected) in cases {
            assert_eq!(format_epoch_seconds(seconds), expected, "at {seconds}");
        }
    }

    #[test]
    fn a_century_that_is_not_a_leap_year_is_handled() {
        // 2100 is divisible by 4 and by 100 but not by 400.
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(2000));
        assert_eq!(days_in_year(2100), 365);
        assert_eq!(days_in_year(2000), 366);
    }

    #[test]
    fn the_system_clock_renders_the_declared_shape() {
        let rendered = SystemClock.now_rfc3339_utc();
        assert_eq!(rendered.len(), 20, "{rendered}");
        assert!(rendered.ends_with('Z'), "{rendered}");
        assert_eq!(
            rendered.as_bytes().get(10).copied(),
            Some(b'T'),
            "{rendered}"
        );
        // The workspace was created well after 2020; a clock that says otherwise is not
        // a clock.
        assert!(rendered.as_str() > "2020-01-01T00:00:00Z", "{rendered}");
    }

    #[test]
    fn a_fixed_clock_never_moves() {
        let clock = FixedClock("2026-01-01T00:00:00Z".to_owned());
        assert_eq!(clock.now_rfc3339_utc(), "2026-01-01T00:00:00Z");
        assert_eq!(clock.now_rfc3339_utc(), clock.now_rfc3339_utc());
    }

    #[test]
    fn a_clock_is_usable_behind_a_trait_object() {
        let clocks: Vec<Box<dyn Clock>> = vec![
            Box::new(SystemClock),
            Box::new(FixedClock("1970-01-01T00:00:00Z".to_owned())),
        ];
        assert_eq!(clocks.len(), 2);
        assert_eq!(clocks[1].now_rfc3339_utc(), "1970-01-01T00:00:00Z");
    }
}
