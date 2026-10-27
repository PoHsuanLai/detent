//! The portal's list as the window reads it: the rows `ListGrants` answers with, made into grants.

use super::grant::{GRANTS_MAX, ScreenGrant};
use std::collections::BTreeSet;

/// A row of `ListGrants`: app id, the portal's name for the app, mode, when it was allowed.
pub type ScreenGrantRow = (String, String, u32, u64);

/// The grants the rows hold. A row that is not one is left out, an app listed twice is listed
/// once, and no more than [`GRANTS_MAX`] are kept. The portal's own name for an app (the second
/// field) is not read: it comes from a desktop entry the app or the person may have written, and
/// an app is named from the id it was proven by (`ProgramBook::look_of`).
pub fn screen_grants_of(rows: &[ScreenGrantRow]) -> Vec<ScreenGrant> {
    let mut seen = BTreeSet::new();
    rows.iter()
        .filter_map(|(app, _, mode, granted)| ScreenGrant::of_wire(app, *mode, *granted))
        .filter(|grant| seen.insert(grant.app.clone()))
        .take(GRANTS_MAX)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ScreenGrantRow, screen_grants_of};
    use crate::screen_sharing::GRANTS_MAX;

    fn row(app: &str, name: &str, mode: u32) -> ScreenGrantRow {
        (app.to_owned(), name.to_owned(), mode, 100)
    }

    fn apps(rows: &[ScreenGrantRow]) -> Vec<String> {
        screen_grants_of(rows)
            .into_iter()
            .map(|g| g.app.0)
            .collect()
    }

    #[test]
    fn a_row_that_is_not_a_grant_is_left_out_and_the_rest_are_read() {
        let rows = [
            row("org.example.Meet", "Meet", 2),
            row("", "Nobody", 2),
            row("org.example.Chat", "Chat", 0),
            row("org.example.Call", "Call", 1),
        ];
        assert_eq!(apps(&rows), ["org.example.Meet", "org.example.Call"]);
    }

    #[test]
    fn an_app_listed_twice_is_listed_once() {
        let rows = [
            row("org.example.Meet", "Meet", 2),
            row("org.example.Meet", "Meet", 1),
        ];
        assert_eq!(apps(&rows), ["org.example.Meet"]);
    }

    #[test]
    fn the_portals_name_for_an_app_is_not_read() {
        let rows = [row("org.evil.Grabber", "System Settings", 2)];
        assert_eq!(apps(&rows), ["org.evil.Grabber"]);
    }

    #[test]
    fn a_list_longer_than_the_portal_keeps_is_cut() {
        let rows: Vec<ScreenGrantRow> = (0..GRANTS_MAX + 40)
            .map(|n| row(&format!("org.example.App{n}"), "App", 2))
            .collect();
        assert_eq!(screen_grants_of(&rows).len(), GRANTS_MAX);
    }
}
