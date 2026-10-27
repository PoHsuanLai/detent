//! Every page the app knows, resolved once from the schemas' `Page` where the catalogue builds a
//! `PageModel`. Nothing else asks "is this the Wi-Fi page": it matches on the kind.

use crate::accounts::{ACCOUNTS_SETUP_SLUG, ModuleId};
use crate::assistant_model::{ASSISTANT_MODEL_NAME, ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG};
use crate::bluetooth::BLUETOOTH_SLUG;
use crate::displays::DISPLAYS_SLUG;
use crate::extensions::EXTENSIONS_SLUG;
use crate::page::PageSlug;
use crate::updates::{SOFTWARE_UPDATE_NAME, SOFTWARE_UPDATE_SLUG};
use crate::wallpaper::WALLPAPER_SLUG;
use crate::wifi::WIFI_SLUG;
use ds_settings::schema::Page;

/// The three daemons the Accounts pane lists accounts from, in the order it lists them.
const ACCOUNT_MODULES: &[ModuleId] = &[ModuleId::Accountd, ModuleId::Inferd, ModuleId::Syncd];

/// The one daemon the Assistant Model page reads: inferd's picker.
const INFERD: &[ModuleId] = &[ModuleId::Inferd];

/// What a page is: one of quire's fixed pages, one of the pages the catalogue declares itself, or
/// a third-party program's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageKind {
    Appearance,
    Dock,
    MouseAndGestures,
    KeyboardAndShortcuts,
    Notifications,
    Spaces,
    Accounts,
    Privacy,
    Power,
    Apps,
    Intelligence,
    Viewer,
    SoftwareUpdate,
    Wallpaper,
    AssistantModel,
    Wifi,
    Bluetooth,
    Displays,
    Extensions,
    /// A third-party program's page, or one this build has never heard of.
    Program,
}

impl PageKind {
    /// The kind of `page`. The one place a page is told apart from the others.
    pub fn of(page: &Page) -> PageKind {
        match page {
            Page::Appearance => PageKind::Appearance,
            Page::Dock => PageKind::Dock,
            Page::MouseAndGestures => PageKind::MouseAndGestures,
            Page::KeyboardAndShortcuts => PageKind::KeyboardAndShortcuts,
            Page::Notifications => PageKind::Notifications,
            Page::Spaces => PageKind::Spaces,
            Page::Accounts => PageKind::Accounts,
            Page::Privacy => PageKind::Privacy,
            Page::Power => PageKind::Power,
            Page::Apps => PageKind::Apps,
            Page::Intelligence => PageKind::Intelligence,
            Page::Viewer => PageKind::Viewer,
            Page::App(id) => PageKind::declared(id),
        }
    }

    fn declared(id: &str) -> PageKind {
        match id {
            SOFTWARE_UPDATE_NAME => PageKind::SoftwareUpdate,
            ASSISTANT_MODEL_NAME => PageKind::AssistantModel,
            WALLPAPER_SLUG => PageKind::Wallpaper,
            WIFI_SLUG => PageKind::Wifi,
            BLUETOOTH_SLUG => PageKind::Bluetooth,
            DISPLAYS_SLUG => PageKind::Displays,
            EXTENSIONS_SLUG => PageKind::Extensions,
            _ => PageKind::Program,
        }
    }

    /// Whether `slug` names the first-run step "Choose an assistant model": a page of its own that
    /// is reached by its slug and never listed.
    pub fn is_assistant_setup(slug: &PageSlug) -> bool {
        slug.0 == ASSISTANT_SETUP_SLUG
    }

    /// Whether `slug` names the first-run step "Sign in to your accounts".
    pub fn is_accounts_setup(slug: &PageSlug) -> bool {
        slug.0 == ACCOUNTS_SETUP_SLUG
    }

    /// The page's stable name, or none for a program's page, which is named by its id.
    pub(crate) fn slug(self) -> Option<&'static str> {
        Some(match self {
            PageKind::Appearance => "appearance",
            PageKind::Dock => "dock",
            PageKind::MouseAndGestures => "mouse_and_gestures",
            PageKind::KeyboardAndShortcuts => "keyboard_and_shortcuts",
            PageKind::Notifications => "notifications",
            PageKind::Spaces => "spaces",
            PageKind::Accounts => "accounts",
            PageKind::Privacy => "privacy",
            PageKind::Power => "power",
            PageKind::Apps => "apps",
            PageKind::Intelligence => "intelligence",
            PageKind::Viewer => "viewer",
            PageKind::SoftwareUpdate => SOFTWARE_UPDATE_SLUG,
            PageKind::Wallpaper => WALLPAPER_SLUG,
            PageKind::AssistantModel => ASSISTANT_MODEL_SLUG,
            PageKind::Wifi => WIFI_SLUG,
            PageKind::Bluetooth => BLUETOOTH_SLUG,
            PageKind::Displays => DISPLAYS_SLUG,
            PageKind::Extensions => EXTENSIONS_SLUG,
            PageKind::Program => return None,
        })
    }

    /// The daemons the page reads while it is shown, each once. The match names every kind, so a
    /// new page cannot be added without deciding what it reads.
    ///
    /// - Accounts: the three modules of its pane.
    /// - Intelligence: inferd (the model picker and "Your computers"), accountd and syncd (the
    ///   accounts the local runtimes and the cloud places read).
    /// - Spaces: accountd, which keeps the registry.
    /// - Assistant Model: inferd, whose picker holds the row.
    /// - Privacy: none. The compositor publishes the rows; Settings looks for it and never
    ///   starts it.
    pub fn modules_read(self) -> &'static [ModuleId] {
        match self {
            PageKind::Accounts | PageKind::Intelligence => ACCOUNT_MODULES,
            PageKind::Spaces => &[ModuleId::Accountd],
            PageKind::AssistantModel => INFERD,
            PageKind::Appearance
            | PageKind::Dock
            | PageKind::MouseAndGestures
            | PageKind::KeyboardAndShortcuts
            | PageKind::Notifications
            | PageKind::Privacy
            | PageKind::Power
            | PageKind::Apps
            | PageKind::Viewer
            | PageKind::SoftwareUpdate
            | PageKind::Wallpaper
            | PageKind::Wifi
            | PageKind::Bluetooth
            | PageKind::Displays
            | PageKind::Extensions
            | PageKind::Program => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageKind;
    use crate::accounts::ModuleId;
    use ds_settings::schema::Page;

    const ALL: [PageKind; 20] = [
        PageKind::Appearance,
        PageKind::Dock,
        PageKind::MouseAndGestures,
        PageKind::KeyboardAndShortcuts,
        PageKind::Notifications,
        PageKind::Spaces,
        PageKind::Accounts,
        PageKind::Privacy,
        PageKind::Power,
        PageKind::Apps,
        PageKind::Intelligence,
        PageKind::Viewer,
        PageKind::SoftwareUpdate,
        PageKind::Wallpaper,
        PageKind::AssistantModel,
        PageKind::Wifi,
        PageKind::Bluetooth,
        PageKind::Displays,
        PageKind::Extensions,
        PageKind::Program,
    ];

    #[test]
    fn a_page_resolves_to_its_kind() {
        let cases = [
            (Page::Appearance, PageKind::Appearance),
            (Page::Intelligence, PageKind::Intelligence),
            (crate::wifi_page(), PageKind::Wifi),
            (crate::bluetooth_page(), PageKind::Bluetooth),
            (crate::displays_page(), PageKind::Displays),
            (crate::extensions_page(), PageKind::Extensions),
            (crate::assistant_model_page(), PageKind::AssistantModel),
            (Page::declared("wallpaper"), PageKind::Wallpaper),
            (Page::App("acme".to_owned()), PageKind::Program),
        ];
        for (page, want) in cases {
            assert_eq!(PageKind::of(&page), want, "{page:?}");
        }
    }

    #[test]
    fn only_a_program_has_no_slug_of_its_own() {
        assert_eq!(PageKind::Program.slug(), None);
        assert_eq!(PageKind::Wifi.slug(), Some("wifi"));
        assert_eq!(PageKind::Wallpaper.slug(), Some("wallpaper"));
    }

    #[test]
    fn the_pages_that_read_a_daemon_name_it() {
        assert_eq!(PageKind::Accounts.modules_read(), ModuleId::ALL);
        assert_eq!(PageKind::Intelligence.modules_read(), ModuleId::ALL);
        assert_eq!(PageKind::Spaces.modules_read(), [ModuleId::Accountd]);
        assert_eq!(PageKind::AssistantModel.modules_read(), [ModuleId::Inferd]);
    }

    #[test]
    fn a_page_that_reads_files_only_starts_nothing() {
        for kind in [
            PageKind::Appearance,
            PageKind::Dock,
            PageKind::MouseAndGestures,
            PageKind::KeyboardAndShortcuts,
            PageKind::Notifications,
            PageKind::Privacy,
            PageKind::Power,
            PageKind::Apps,
            PageKind::Viewer,
            PageKind::Program,
        ] {
            assert!(kind.modules_read().is_empty(), "{kind:?}");
        }
    }

    #[test]
    fn every_daemon_detent_reads_is_named_by_some_page() {
        for module in ModuleId::ALL {
            assert!(
                ALL.iter().any(|kind| kind.modules_read().contains(&module)),
                "{module:?}"
            );
        }
    }

    #[test]
    fn a_page_names_a_daemon_once() {
        for kind in ALL {
            let mut seen = kind.modules_read().to_vec();
            seen.sort();
            seen.dedup();
            assert_eq!(seen.len(), kind.modules_read().len(), "{kind:?}");
        }
    }
}
