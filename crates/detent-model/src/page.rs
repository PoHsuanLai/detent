//! The fixed page order and each page's slug (design/22 section 9.3).

use crate::updates::{SLUG as SOFTWARE_UPDATE, is_software_update};
use ds_settings::schema::Page;
use std::cmp::Ordering;

/// A page's stable name: what the sidebar keys on and `--page` takes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PageSlug(pub String);

impl PageSlug {
    /// The slug of `page`: the fixed pages by their snake-case name, a third-party page by
    /// `app:<id>`.
    pub fn of(page: &Page) -> PageSlug {
        PageSlug(match page {
            Page::Appearance => "appearance".to_owned(),
            Page::Dock => "dock".to_owned(),
            Page::MouseAndGestures => "mouse_and_gestures".to_owned(),
            Page::KeyboardAndShortcuts => "keyboard_and_shortcuts".to_owned(),
            Page::Notifications => "notifications".to_owned(),
            Page::Spaces => "spaces".to_owned(),
            Page::Accounts => "accounts".to_owned(),
            Page::Power => "power".to_owned(),
            Page::Apps => "apps".to_owned(),
            Page::Intelligence => "intelligence".to_owned(),
            Page::Viewer => "viewer".to_owned(),
            Page::App(_) if is_software_update(page) => SOFTWARE_UPDATE.to_owned(),
            Page::App(id) => format!("app:{id}"),
        })
    }
}

/// Where `page` sits in the sidebar: the fixed pages in section 9.3's order, Software Update, then
/// every third-party page by app id.
fn place(page: &Page) -> (u8, &str) {
    match page {
        Page::Appearance => (0, ""),
        Page::Dock => (1, ""),
        Page::MouseAndGestures => (2, ""),
        Page::KeyboardAndShortcuts => (3, ""),
        Page::Notifications => (4, ""),
        Page::Spaces => (5, ""),
        Page::Accounts => (6, ""),
        Page::Power => (7, ""),
        Page::Apps => (8, ""),
        Page::Intelligence => (9, ""),
        Page::Viewer => (10, ""),
        Page::App(_) if is_software_update(page) => (11, ""),
        Page::App(id) => (12, id),
    }
}

/// The sidebar order of two pages.
pub(crate) fn sidebar_order(a: &Page, b: &Page) -> Ordering {
    place(a).cmp(&place(b))
}

#[cfg(test)]
mod tests {
    use super::{PageSlug, sidebar_order};
    use ds_settings::schema::Page;

    #[test]
    fn pages_sort_in_the_documented_order() {
        let mut pages = vec![
            Page::App("zed".to_owned()),
            Page::Viewer,
            Page::Intelligence,
            Page::Apps,
            Page::Power,
            Page::App("acme".to_owned()),
            Page::Accounts,
            Page::Spaces,
            Page::Notifications,
            Page::KeyboardAndShortcuts,
            Page::MouseAndGestures,
            Page::Dock,
            Page::Appearance,
        ];
        pages.sort_by(sidebar_order);
        let slugs: Vec<String> = pages.iter().map(|p| PageSlug::of(p).0).collect();
        assert_eq!(
            slugs,
            [
                "appearance",
                "dock",
                "mouse_and_gestures",
                "keyboard_and_shortcuts",
                "notifications",
                "spaces",
                "accounts",
                "power",
                "apps",
                "intelligence",
                "viewer",
                "app:acme",
                "app:zed"
            ]
        );
    }
}
