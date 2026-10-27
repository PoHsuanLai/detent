//! How a page opens: most pages start with their first card; the overview pages start with a hero,
//! the page's tile, its name and one sentence on what it is for.

use crate::page_kind::PageKind;

/// The overview pages that open with a hero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hero {
    /// Privacy & Security.
    Privacy,
    /// Accounts.
    Accounts,
    /// Intelligence.
    Intelligence,
}

/// How a page opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    /// With its first card.
    Plain,
    /// With a hero.
    Hero(Hero),
}

impl PageKind {
    /// How a page of this kind opens.
    pub fn head(self) -> Head {
        match self {
            PageKind::Privacy => Head::Hero(Hero::Privacy),
            PageKind::Accounts => Head::Hero(Hero::Accounts),
            PageKind::Intelligence => Head::Hero(Hero::Intelligence),
            PageKind::Appearance
            | PageKind::Dock
            | PageKind::MouseAndGestures
            | PageKind::KeyboardAndShortcuts
            | PageKind::Notifications
            | PageKind::Spaces
            | PageKind::Power
            | PageKind::Apps
            | PageKind::Viewer
            | PageKind::SoftwareUpdate
            | PageKind::Wallpaper
            | PageKind::AssistantModel
            | PageKind::Wifi
            | PageKind::Bluetooth
            | PageKind::Displays
            | PageKind::Extensions
            | PageKind::Program => Head::Plain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Head, Hero};
    use crate::page_kind::PageKind;

    #[test]
    fn only_the_overview_pages_open_with_a_hero() {
        assert_eq!(PageKind::Privacy.head(), Head::Hero(Hero::Privacy));
        assert_eq!(PageKind::Accounts.head(), Head::Hero(Hero::Accounts));
        assert_eq!(
            PageKind::Intelligence.head(),
            Head::Hero(Hero::Intelligence)
        );
        for kind in [
            PageKind::Wifi,
            PageKind::Bluetooth,
            PageKind::Dock,
            PageKind::Program,
        ] {
            assert_eq!(kind.head(), Head::Plain, "{kind:?}");
        }
    }
}
