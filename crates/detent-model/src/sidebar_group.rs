//! How the sidebar sets pages apart: the group each kind of page sits in and its place inside the
//! group (both in the page's row). The window draws the groups with a gap between them; it
//! decides neither.

use crate::catalogue::PageModel;

/// A run of pages the sidebar sets apart from the next with a gap, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SidebarGroup {
    /// Wi-Fi and Bluetooth.
    Connection,
    /// Notifications.
    Alerts,
    /// How the desktop looks, the assistant, and what programs may reach.
    Look,
    /// The desktop, its screens, its pictures and its Spaces.
    Desktop,
    /// Power and updates.
    System,
    /// The person's accounts.
    Accounts,
    /// Keyboard and pointing devices.
    Input,
    /// Every page a program brings.
    Programs,
}

/// `pages` in groups, each group in its own order; a group with no page is left out. Pages that
/// share a place (a program's pages) keep the order they came in.
pub fn sidebar_groups<'a>(pages: &[&'a PageModel]) -> Vec<Vec<&'a PageModel>> {
    let mut sorted: Vec<&PageModel> = pages.to_vec();
    sorted.sort_by_key(|page| page.kind.traits().slot());
    let mut groups: Vec<Vec<&PageModel>> = Vec::new();
    let mut last = None;
    for page in sorted {
        let group = page.kind.traits().group;
        if last != Some(group) {
            groups.push(Vec::new());
            last = Some(group);
        }
        if let Some(open) = groups.last_mut() {
            open.push(page);
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::{SidebarGroup, sidebar_groups};
    use crate::catalogue::PageModel;
    use crate::page::PageSlug;
    use crate::page_kind::PageKind;
    use ds_settings::schema::Page;

    fn model(page: Page) -> PageModel {
        PageModel {
            slug: PageSlug::of(&page),
            kind: PageKind::of(&page),
            title: String::new(),
            page,
            sections: Vec::new(),
            advanced: Vec::new(),
        }
    }

    fn all() -> Vec<PageModel> {
        [
            Page::App("zed".to_owned()),
            Page::Viewer,
            crate::updates::software_update_page(),
            Page::MouseAndGestures,
            Page::KeyboardAndShortcuts,
            Page::Accounts,
            Page::Power,
            Page::Spaces,
            Page::declared("wallpaper"),
            crate::displays_page(),
            Page::Dock,
            Page::Privacy,
            crate::extensions_page(),
            crate::assistant_model_page(),
            Page::Intelligence,
            Page::Appearance,
            Page::Notifications,
            crate::bluetooth_page(),
            crate::wifi_page(),
        ]
        .into_iter()
        .map(model)
        .collect()
    }

    #[test]
    fn pages_come_in_the_groups_and_order_the_sidebar_draws() {
        let pages = all();
        let refs: Vec<&PageModel> = pages.iter().collect();
        let drawn: Vec<Vec<String>> = sidebar_groups(&refs)
            .iter()
            .map(|group| group.iter().map(|page| page.slug.0.clone()).collect())
            .collect();
        let expected: &[&[&str]] = &[
            &["wifi", "bluetooth"],
            &["notifications"],
            &[
                "appearance",
                "intelligence",
                "assistant-model",
                "extensions",
                "privacy",
            ],
            &["dock", "displays", "wallpaper", "spaces"],
            &["power", "software-update"],
            &["accounts"],
            &["keyboard_and_shortcuts", "mouse_and_gestures"],
            &["app:zed", "viewer"],
        ];
        assert_eq!(drawn, expected);
    }

    #[test]
    fn a_group_with_no_page_leaves_no_gap() {
        let pages = [model(Page::Power), model(Page::Notifications)];
        let refs: Vec<&PageModel> = pages.iter().collect();
        assert_eq!(sidebar_groups(&refs).len(), 2);
        assert_eq!(PageKind::Power.traits().group, SidebarGroup::System);
    }
}
