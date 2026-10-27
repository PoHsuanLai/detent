//! The screen sharing list's words: the line under an app.

use super::updates::age_words;
use detent_model::{ScreenGrant, ShareLength, UnixSeconds};

/// The line under an app: when it was allowed and for how long, such as "Allowed 3 days ago, until
/// you remove it".
pub(crate) fn grant_line(grant: &ScreenGrant, now: UnixSeconds) -> String {
    let age = age_words(now, grant.granted);
    match grant.length {
        ShareLength::UntilRemoved => format!("Allowed {age}, until you remove it"),
        ShareLength::WhileRunning => format!("Allowed {age}, while it is open"),
    }
}

/// The words "Screen sharing".
pub(crate) const SCREEN_SHARING: &str = "Screen sharing";

/// The words "No apps can share your screen without asking".
pub(crate) const NO_APPS_CAN_SHARE_YOUR_SCREEN: &str =
    "No apps can share your screen without asking";

/// The words "When you let an app share your screen without asking again, it appears here.".
pub(crate) const WHEN_YOU_LET_AN_APP_SHARE: &str =
    "When you let an app share your screen without asking again, it appears here.";

/// The words "Remove {name}".
pub(crate) fn remove(name: impl std::fmt::Display) -> String {
    format!("Remove {name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::ProgramSubject;

    const NOW: UnixSeconds = UnixSeconds(1_800_000_000);
    const HOUR: i64 = 3600;
    const DAY: i64 = 24 * HOUR;

    fn grant(length: ShareLength, ago: i64) -> ScreenGrant {
        ScreenGrant {
            app: ProgramSubject("org.example.Meet".to_owned()),
            length,
            granted: UnixSeconds(NOW.0 - ago),
        }
    }

    #[test]
    fn the_line_says_when_an_app_was_allowed_and_for_how_long() {
        let cases = [
            (
                ShareLength::UntilRemoved,
                5,
                "Allowed just now, until you remove it",
            ),
            (
                ShareLength::UntilRemoved,
                3 * HOUR,
                "Allowed 3 hours ago, until you remove it",
            ),
            (
                ShareLength::WhileRunning,
                DAY + HOUR,
                "Allowed yesterday, while it is open",
            ),
            (
                ShareLength::UntilRemoved,
                12 * DAY,
                "Allowed 12 days ago, until you remove it",
            ),
        ];
        for (length, ago, want) in cases {
            assert_eq!(grant_line(&grant(length, ago), NOW), want);
        }
    }
}
