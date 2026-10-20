//! The fixed page order and each page's slug (design/22 section 9.3).

use crate::assistant_model::{ASSISTANT_MODEL_SLUG, is_assistant_model};
use crate::bluetooth::{BLUETOOTH_SLUG, is_bluetooth};
use crate::displays::{DISPLAYS_SLUG, is_displays};
use crate::updates::{SLUG as SOFTWARE_UPDATE, is_software_update};
use crate::wallpaper::{SLUG as WALLPAPER, is_wallpaper};
use crate::wifi::{WIFI_SLUG, is_wifi};
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
            Page::Privacy => "privacy".to_owned(),
            Page::Power => "power".to_owned(),
            Page::Apps => "apps".to_owned(),
            Page::Intelligence => "intelligence".to_owned(),
            Page::Viewer => "viewer".to_owned(),
            Page::App(_) if is_software_update(page) => SOFTWARE_UPDATE.to_owned(),
            Page::App(_) if is_wallpaper(page) => WALLPAPER.to_owned(),
            Page::App(_) if is_assistant_model(page) => ASSISTANT_MODEL_SLUG.to_owned(),
            Page::App(_) if is_wifi(page) => WIFI_SLUG.to_owned(),
            Page::App(_) if is_bluetooth(page) => BLUETOOTH_SLUG.to_owned(),
            Page::App(_) if is_displays(page) => DISPLAYS_SLUG.to_owned(),
            Page::App(id) => format!("app:{id}"),
        })
    }
}

/// Where `page` sits in the sidebar: Wi-Fi and Bluetooth first, as on a Mac, then the fixed pages in
/// section 9.3's order with Displays and Wallpaper after Desktop & Dock and Assistant Model right
/// after Intelligence, Software Update, then every third-party page by app id.
fn place(page: &Page) -> (u8, &str) {
    match page {
        Page::App(_) if is_wifi(page) => (0, ""),
        Page::App(_) if is_bluetooth(page) => (1, ""),
        Page::Appearance => (2, ""),
        Page::Dock => (3, ""),
        Page::App(_) if is_displays(page) => (4, ""),
        Page::App(_) if is_wallpaper(page) => (5, ""),
        Page::MouseAndGestures => (6, ""),
        Page::KeyboardAndShortcuts => (7, ""),
        Page::Notifications => (8, ""),
        Page::Spaces => (9, ""),
        Page::Accounts => (10, ""),
        Page::Privacy => (11, ""),
        Page::Power => (12, ""),
        Page::Apps => (13, ""),
        Page::Intelligence => (14, ""),
        Page::App(_) if is_assistant_model(page) => (15, ""),
        Page::Viewer => (16, ""),
        Page::App(_) if is_software_update(page) => (17, ""),
        Page::App(id) => (18, id),
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
            crate::displays_page(),
            crate::bluetooth_page(),
            crate::wifi_page(),
            Page::App("zed".to_owned()),
            Page::Viewer,
            Page::Intelligence,
            Page::Apps,
            Page::Power,
            Page::App("acme".to_owned()),
            Page::Accounts,
            Page::Privacy,
            Page::Spaces,
            Page::Notifications,
            Page::KeyboardAndShortcuts,
            Page::MouseAndGestures,
            Page::Dock,
            Page::declared("wallpaper"),
            Page::Appearance,
        ];
        pages.sort_by(sidebar_order);
        let slugs: Vec<String> = pages.iter().map(|p| PageSlug::of(p).0).collect();
        assert_eq!(
            slugs,
            [
                "wifi",
                "bluetooth",
                "appearance",
                "dock",
                "displays",
                "wallpaper",
                "mouse_and_gestures",
                "keyboard_and_shortcuts",
                "notifications",
                "spaces",
                "accounts",
                "privacy",
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
