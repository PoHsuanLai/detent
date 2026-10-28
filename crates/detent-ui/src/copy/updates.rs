//! What the Software Update page says: the standing, the actions, the sizes and the ages.

use detent_model::{
    ByteCount, Checked, Downloaded, ReleaseNotes, UnixSeconds, UpdateAction, UpdateStanding,
    UpdatesNotice,
};

/// The sizes a download is told in, largest first (decimal, as the Mac tells them).
const UNITS: &[(u64, &str)] = &[(1_000_000_000, "GB"), (1_000_000, "MB"), (1_000, "KB")];

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const TWO_DAYS: i64 = 2 * DAY;

/// What the daily check is, as the page tells it.
pub(crate) fn automatic() -> &'static str {
    "Quire checks once a day and asks before it installs anything."
}

/// The version the row is about: "Quire 0.3.1", or "Quire" when none is known.
pub(crate) fn title(version: Option<&str>) -> String {
    version.map_or_else(|| "Quire".to_owned(), |version| format!("Quire {version}"))
}

/// What is true of the version, under the title.
pub(crate) fn standing(standing: &UpdateStanding) -> String {
    match standing {
        UpdateStanding::UpToDate => "Your computer is up to date".to_owned(),
        UpdateStanding::Checking => "Checking for updates\u{2026}".to_owned(),
        UpdateStanding::Available(ByteCount(0)) => "Update available".to_owned(),
        UpdateStanding::Available(size) => {
            format!("Update available \u{b7} {}", size_words(*size))
        }
        UpdateStanding::Downloading => "Downloading\u{2026}".to_owned(),
        UpdateStanding::Preparing => "Preparing to install\u{2026}".to_owned(),
        UpdateStanding::Installing => "Installing\u{2026}".to_owned(),
        UpdateStanding::Installed => "Log out to finish the update".to_owned(),
        UpdateStanding::Failed(problem) => problem
            .clone()
            .unwrap_or_else(|| "The update did not finish. Try again in a moment.".to_owned()),
    }
}

/// The line about the last check.
pub(crate) fn checked(checked: Checked) -> String {
    match checked {
        Checked::Never => "Not checked yet".to_owned(),
        Checked::Ago(seconds) => format!("Last checked {}", elapsed_words(seconds)),
    }
}

/// The word of caution under the row, when there is one.
pub(crate) fn notice(notice: UpdatesNotice) -> Option<&'static str> {
    match notice {
        UpdatesNotice::Nothing => None,
        UpdatesNotice::NewerVersion => {
            Some("A newer version arrived. Check what's new before installing.")
        }
    }
}

/// The words on an action's button.
pub(crate) fn action(action: UpdateAction) -> &'static str {
    match action {
        UpdateAction::Check { .. } => "Check for Updates",
        UpdateAction::MoreInfo => "More Info\u{2026}",
        UpdateAction::UpdateNow => "Update Now",
        UpdateAction::TryAgain => "Try Again",
        UpdateAction::LogOut => "Log Out\u{2026}",
    }
}

/// "12.3 MB of 45.6 MB".
pub(crate) fn downloaded(downloaded: Downloaded) -> String {
    format!(
        "{} of {}",
        size_words(downloaded.done),
        size_words(downloaded.total)
    )
}

/// The notes as the sheet shows them: the release's own, or a line that there are none.
pub(crate) fn notes_body(notes: &ReleaseNotes) -> String {
    notes
        .text
        .clone()
        .unwrap_or_else(|| "No release notes came with this update.".to_owned())
}

/// `size` as a person reads it: "45.6 MB", "345 MB", "812 bytes".
fn size_words(size: ByteCount) -> String {
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
pub(crate) fn age_words(now: UnixSeconds, then: UnixSeconds) -> String {
    elapsed_words(now.0.saturating_sub(then.0))
}

/// `seconds` ago, in the same words as [`age_words`], for an age a service already measured.
pub(crate) fn elapsed_words(seconds: i64) -> String {
    match seconds {
        ..MINUTE => "just now".to_owned(),
        MINUTE..HOUR => format!("{} ago", counted(seconds / MINUTE, "minute")),
        HOUR..DAY => format!("{} ago", counted(seconds / HOUR, "hour")),
        DAY..TWO_DAYS => "yesterday".to_owned(),
        _ => format!("{} ago", counted(seconds / DAY, "day")),
    }
}

/// The words "Software Update".
pub(crate) const SOFTWARE_UPDATE: &str = "Software Update";

/// The words "Automatic updates".
pub(crate) const AUTOMATIC_UPDATES: &str = "Automatic updates";

/// The words "Installing".
pub(crate) const INSTALLING: &str = "Installing";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_update_page_says_what_each_fact_means() {
        // `(bytes, the words)`.
        const SIZES: &[(u64, &str)] = &[
            (0, "0 bytes"),
            (1, "1 byte"),
            (812, "812 bytes"),
            (1_000, "1.0 KB"),
            (45_600_000, "45.6 MB"),
            (345_000_000, "345 MB"),
            (1_240_000_000, "1.2 GB"),
        ];
        for (bytes, want) in SIZES {
            assert_eq!(size_words(ByteCount(*bytes)), *want, "{bytes}");
        }

        // `(seconds ago, the words)`; a negative age is a clock a little ahead.
        const AGES: &[(i64, &str)] = &[
            (-30, "just now"),
            (0, "just now"),
            (59, "just now"),
            (60, "1 minute ago"),
            (300, "5 minutes ago"),
            (3_600, "1 hour ago"),
            (7_200, "2 hours ago"),
            (86_400, "yesterday"),
            (172_799, "yesterday"),
            (172_800, "2 days ago"),
            (864_000, "10 days ago"),
        ];
        for (ago, want) in AGES {
            assert_eq!(elapsed_words(*ago), *want, "{ago} seconds");
        }

        assert_eq!(title(Some("0.3.1")), "Quire 0.3.1");
        assert_eq!(title(None), "Quire");

        let standings = [
            (UpdateStanding::UpToDate, "Your computer is up to date"),
            (UpdateStanding::Checking, "Checking for updates\u{2026}"),
            (UpdateStanding::Available(ByteCount(0)), "Update available"),
            (
                UpdateStanding::Available(ByteCount(45_600_000)),
                "Update available \u{b7} 45.6 MB",
            ),
            (UpdateStanding::Downloading, "Downloading\u{2026}"),
            (UpdateStanding::Preparing, "Preparing to install\u{2026}"),
            (UpdateStanding::Installing, "Installing\u{2026}"),
            (UpdateStanding::Installed, "Log out to finish the update"),
            (
                UpdateStanding::Failed(Some("There is no network.".to_owned())),
                "There is no network.",
            ),
            (
                UpdateStanding::Failed(None),
                "The update did not finish. Try again in a moment.",
            ),
        ];
        for (fact, want) in standings {
            assert_eq!(standing(&fact), want);
        }

        assert_eq!(checked(Checked::Never), "Not checked yet");
        assert_eq!(checked(Checked::Ago(300)), "Last checked 5 minutes ago");
        assert_eq!(notice(UpdatesNotice::Nothing), None);
        assert_eq!(
            notice(UpdatesNotice::NewerVersion),
            Some("A newer version arrived. Check what's new before installing.")
        );

        // `(action, the label)`.
        const ACTIONS: &[(UpdateAction, &str)] = &[
            (UpdateAction::Check { running: false }, "Check for Updates"),
            (UpdateAction::Check { running: true }, "Check for Updates"),
            (UpdateAction::MoreInfo, "More Info\u{2026}"),
            (UpdateAction::UpdateNow, "Update Now"),
            (UpdateAction::TryAgain, "Try Again"),
            (UpdateAction::LogOut, "Log Out\u{2026}"),
        ];
        for (fact, want) in ACTIONS {
            assert_eq!(action(*fact), *want);
        }

        assert_eq!(
            downloaded(Downloaded {
                done: ByteCount(11_400_000),
                total: ByteCount(45_600_000),
            }),
            "11.4 MB of 45.6 MB"
        );
        let notes = |text: Option<&str>| ReleaseNotes {
            version: "0.4.0".to_owned(),
            text: text.map(str::to_owned),
        };
        assert_eq!(notes_body(&notes(Some("Faster start."))), "Faster start.");
        assert_eq!(
            notes_body(&notes(None)),
            "No release notes came with this update."
        );
        assert!(!automatic().is_empty());
    }
}
