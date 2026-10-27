//! The Extensions page's words and the Look menu's note on a choice that is gone.

use detent_model::{ExtensionKind, ExtensionRow, ExtensionStanding, ExtensionTrouble};

/// Under a stored choice that nothing installed has now (removed, or turned off).
pub(crate) const NOT_INSTALLED: &str = "Not installed, or turned off";

/// Where the switch would be for an Extension that cannot be used.
pub(crate) const NOT_AVAILABLE: &str = "Not Available";

/// The page when nothing is installed: the title.
pub(crate) const EXTENSIONS_EMPTY_TITLE: &str = "No Extensions";

/// The page when nothing is installed: what to expect.
pub(crate) const EXTENSIONS_EMPTY_HELP: &str = "Looks and other additions you install appear here.";

/// The heading over a group of Extensions.
pub(crate) fn group_title(kind: ExtensionKind) -> &'static str {
    match kind {
        ExtensionKind::Looks => "Looks",
        ExtensionKind::FileIcons => "File Icons",
        ExtensionKind::Other => "Other",
        ExtensionKind::Unusable => "Can\u{2019}t Be Used",
    }
}

/// Why an Extension cannot be used, in plain words.
fn trouble_line(trouble: &ExtensionTrouble) -> &str {
    match trouble {
        ExtensionTrouble::Unreadable => "This Extension\u{2019}s description could not be read",
        ExtensionTrouble::Reported(sentence) => sentence,
    }
}

/// The line under an Extension's name: why it cannot be used, else what it says of itself.
pub(crate) fn extension_line(row: &ExtensionRow) -> Option<&str> {
    match &row.standing {
        ExtensionStanding::Unusable(trouble) => Some(trouble_line(trouble)),
        ExtensionStanding::On | ExtensionStanding::Off => row.summary.as_deref(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::ExtensionKeep;

    fn row(standing: ExtensionStanding) -> ExtensionRow {
        ExtensionRow {
            id: "a.b".to_owned(),
            name: "Paper".to_owned(),
            summary: Some("Warm paper".to_owned()),
            provides: vec!["look".to_owned()],
            standing,
            keep: ExtensionKeep::Free,
        }
    }

    #[test]
    fn a_group_has_its_heading() {
        let cases = [
            (ExtensionKind::Looks, "Looks"),
            (ExtensionKind::FileIcons, "File Icons"),
            (ExtensionKind::Other, "Other"),
            (ExtensionKind::Unusable, "Can\u{2019}t Be Used"),
        ];
        for (kind, want) in cases {
            assert_eq!(group_title(kind), want);
        }
    }

    #[test]
    fn the_reason_replaces_the_summary_of_an_extension_that_cannot_be_used() {
        let cases = [
            (ExtensionStanding::On, Some("Warm paper")),
            (ExtensionStanding::Off, Some("Warm paper")),
            (
                ExtensionStanding::Unusable(ExtensionTrouble::Unreadable),
                Some("This Extension\u{2019}s description could not be read"),
            ),
            (
                ExtensionStanding::Unusable(ExtensionTrouble::Reported("Torn".to_owned())),
                Some("Torn"),
            ),
        ];
        for (standing, want) in cases {
            assert_eq!(extension_line(&row(standing)), want);
        }
    }
}
