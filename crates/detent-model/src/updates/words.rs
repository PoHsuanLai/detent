//! The page's numbers in words: a size, and how long ago something was.

use super::model::{ByteCount, UnixSeconds};

/// The sizes a download is told in, largest first (decimal, as the Mac tells them).
const UNITS: &[(u64, &str)] = &[(1_000_000_000, "GB"), (1_000_000, "MB"), (1_000, "KB")];

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const TWO_DAYS: i64 = 2 * DAY;

/// `size` as a person reads it: "45.6 MB", "345 MB", "812 bytes".
pub(super) fn size_words(size: ByteCount) -> String {
    let bytes = size.0;
    let Some((unit, name)) = UNITS.iter().copied().find(|(unit, _)| bytes >= *unit) else {
        return counted(i64::try_from(bytes).unwrap_or(i64::MAX), "byte");
    };
    let unit = u128::from(unit);
    let tenths = (u128::from(bytes) * 10 + unit / 2) / unit;
    if tenths >= 1000 {
        format!("{} {name}", tenths / 10)
    } else {
        format!("{}.{} {name}", tenths / 10, tenths % 10)
    }
}

/// `count` of `unit`, plural as it needs to be: "1 minute", "5 minutes".
fn counted(count: i64, unit: &str) -> String {
    if count == 1 {
        format!("{count} {unit}")
    } else {
        format!("{count} {unit}s")
    }
}

/// How long before `now` the moment `then` was: "just now", "5 minutes ago", "yesterday". A
/// moment that is not yet past (the clocks differ a little) is "just now".
pub(super) fn age_words(now: UnixSeconds, then: UnixSeconds) -> String {
    let seconds = now.0.saturating_sub(then.0);
    match seconds {
        ..MINUTE => "just now".to_owned(),
        MINUTE..HOUR => format!("{} ago", counted(seconds / MINUTE, "minute")),
        HOUR..DAY => format!("{} ago", counted(seconds / HOUR, "hour")),
        DAY..TWO_DAYS => "yesterday".to_owned(),
        _ => format!("{} ago", counted(seconds / DAY, "day")),
    }
}
