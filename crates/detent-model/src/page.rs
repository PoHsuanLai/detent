//! Each page's slug (design/22 section 9.3) and where quire's page table puts it in the sidebar.

use crate::assistant_model::{ASSISTANT_MODEL_SLUG, is_assistant_model};
use crate::bluetooth::{BLUETOOTH_SLUG, is_bluetooth};
use crate::displays::{DISPLAYS_SLUG, is_displays};
use crate::extensions::{EXTENSIONS_SLUG, is_extensions};
use crate::updates::{SLUG as SOFTWARE_UPDATE, is_software_update};
use crate::wallpaper::{SLUG as WALLPAPER, is_wallpaper};
use crate::wifi::{WIFI_SLUG, is_wifi};
use ds_settings::schema::{Page, PageOrder, PageTable};

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
            Page::App(_) if is_extensions(page) => EXTENSIONS_SLUG.to_owned(),
            Page::App(id) => format!("app:{id}"),
        })
    }
}

/// Where `page` sits in the sidebar, by quire's page table: its declared order, or none when the
/// table declares it hidden. A page nothing declares (a third-party program's) comes after every
/// declared one, by id.
pub(crate) fn sidebar_place(table: &PageTable, page: &Page) -> Option<(PageOrder, String)> {
    match table.declared(page) {
        Some(decl) => table
            .sidebar()
            .any(|listed| listed.id == decl.id)
            .then(|| (decl.order, String::new())),
        None => Some((PageOrder(u16::MAX), page.id().0)),
    }
}

/// The sidebar order of two pages (what the order tests sort by).
#[cfg(test)]
pub(crate) fn sidebar_order(table: &PageTable, a: &Page, b: &Page) -> std::cmp::Ordering {
    sidebar_place(table, a).cmp(&sidebar_place(table, b))
}

#[cfg(test)]
mod tests {
    use super::{PageSlug, sidebar_order};
    use ds_settings::schema::{Page, PageTable};

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
            crate::extensions_page(),
            Page::Appearance,
        ];
        let table = PageTable::built_in();
        pages.sort_by(|a, b| sidebar_order(&table, a, b));
        let slugs: Vec<String> = pages.iter().map(|p| PageSlug::of(p).0).collect();
        assert_eq!(
            slugs,
            [
                "wifi",
                "bluetooth",
                "notifications",
                "appearance",
                "extensions",
                "intelligence",
                "privacy",
                "dock",
                "displays",
                "wallpaper",
                "spaces",
                "power",
                "accounts",
                "keyboard_and_shortcuts",
                "mouse_and_gestures",
                "apps",
                "viewer",
                "app:acme",
                "app:zed"
            ]
        );
    }
}
