//! How the sidebar sets pages apart: the group each kind of page sits in, its place inside the
//! group, and the ground its tile is painted on. The window draws the groups with a gap between
//! them and the grounds as colours; it decides neither.

use crate::catalogue::PageModel;
use crate::page_kind::PageKind;

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

/// The ground a page's tile is painted on: a name, which the window turns into a colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ground {
    /// Connection, screens, privacy and accounts.
    Blue,
    /// Notifications.
    Red,
    /// Appearance, and the Dock.
    Graphite,
    /// Wallpaper.
    Cyan,
    /// Power.
    Green,
    /// The plain system pages: updates, input devices, extensions, programs.
    Grey,
    /// The assistant and Spaces.
    Indigo,
}

impl PageKind {
    /// The group the page sits in and its place inside the group.
    fn slot(self) -> (SidebarGroup, u8) {
        use SidebarGroup::{Accounts, Alerts, Connection, Desktop, Input, Look, Programs, System};
        match self {
            PageKind::Wifi => (Connection, 0),
            PageKind::Bluetooth => (Connection, 1),
            PageKind::Notifications => (Alerts, 0),
            PageKind::Appearance => (Look, 0),
            PageKind::Intelligence => (Look, 1),
            PageKind::AssistantModel => (Look, 2),
            PageKind::Extensions => (Look, 3),
            PageKind::Privacy => (Look, 4),
            PageKind::Dock => (Desktop, 0),
            PageKind::Displays => (Desktop, 1),
            PageKind::Wallpaper => (Desktop, 2),
            PageKind::Spaces => (Desktop, 3),
            PageKind::Power => (System, 0),
            PageKind::SoftwareUpdate => (System, 1),
            PageKind::Accounts => (Accounts, 0),
            PageKind::KeyboardAndShortcuts => (Input, 0),
            PageKind::MouseAndGestures => (Input, 1),
            PageKind::Apps | PageKind::Viewer | PageKind::Program => (Programs, 0),
        }
    }

    /// The group the page sits in.
    pub fn group(self) -> SidebarGroup {
        self.slot().0
    }

    /// The ground of the page's tile.
    pub fn ground(self) -> Ground {
        match self {
            PageKind::Wifi
            | PageKind::Bluetooth
            | PageKind::Displays
            | PageKind::Privacy
            | PageKind::Accounts => Ground::Blue,
            PageKind::Notifications => Ground::Red,
            PageKind::Appearance | PageKind::Dock => Ground::Graphite,
            PageKind::Wallpaper => Ground::Cyan,
            PageKind::Power => Ground::Green,
            PageKind::Intelligence | PageKind::AssistantModel | PageKind::Spaces => Ground::Indigo,
            PageKind::SoftwareUpdate
            | PageKind::Extensions
            | PageKind::KeyboardAndShortcuts
            | PageKind::MouseAndGestures
            | PageKind::Apps
            | PageKind::Viewer
            | PageKind::Program => Ground::Grey,
        }
    }
}

/// `pages` in groups, each group in its own order; a group with no page is left out. Pages that
/// share a place (a program's pages) keep the order they came in.
pub fn sidebar_groups<'a>(pages: &[&'a PageModel]) -> Vec<Vec<&'a PageModel>> {
    let mut sorted: Vec<&PageModel> = pages.to_vec();
    sorted.sort_by_key(|page| page.kind.slot());
    let mut groups: Vec<Vec<&PageModel>> = Vec::new();
    let mut last = None;
    for page in sorted {
        let group = page.kind.group();
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
    use super::{Ground, SidebarGroup, sidebar_groups};
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
        assert_eq!(PageKind::Power.group(), SidebarGroup::System);
    }

    #[test]
    fn the_grounds_follow_the_kind() {
        assert_eq!(PageKind::Wifi.ground(), Ground::Blue);
        assert_eq!(PageKind::Notifications.ground(), Ground::Red);
        assert_eq!(PageKind::Program.ground(), Ground::Grey);
    }
}
