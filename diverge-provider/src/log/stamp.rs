//! The time, as a log writes it.

use std::time::{SystemTime, UNIX_EPOCH};

/// Now, in UTC, to the millisecond, as `2026-11-19T16:25:30.123Z`
/// where `compact` is `false` and as `20261119T162530.123Z` where it
/// is `true`: the first is how a line begins, the second how a file is
/// named, which holds no `:`. Worked from the seconds since the epoch,
/// with no calendar asked: the days are turned into a year, a month
/// and a day by the civil-from-days rule of the proleptic Gregorian
/// calendar.
pub fn stamp(compact: bool) -> String {
    let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let (seconds, millis) = (since.as_secs(), since.subsec_millis());
    let (days, of_day) = (seconds / 86_400, seconds % 86_400);
    let (hour, minute, second) = (of_day / 3600, of_day % 3600 / 60, of_day % 60);
    // Days since 0000-03-01, in eras of 400 years.
    let shifted = days as i64 + 719_468;
    let era = shifted.div_euclid(146_097);
    let of_era = shifted.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let day_of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    if compact {
        format!("{year:04}{month:02}{day:02}T{hour:02}{minute:02}{second:02}.{millis:03}Z")
    } else {
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
    }
}
