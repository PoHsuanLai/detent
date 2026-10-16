//! Which live daemons a page reads: the one table the window starts them from when the page is
//! shown. A page that is not here reads files only.

use crate::accounts::ModuleId;
use crate::assistant_model::is_assistant_model;
use ds_settings::schema::Page;

/// The three daemons the Accounts pane lists accounts from, in the order it lists them.
const ACCOUNT_MODULES: &[ModuleId] = &[ModuleId::Accountd, ModuleId::Inferd, ModuleId::Syncd];

/// The one daemon the Assistant Model page reads: inferd's picker.
const INFERD: &[ModuleId] = &[ModuleId::Inferd];

/// The daemons `page` reads while it is shown, each once. The match names every page, so a new
/// page cannot be added without deciding what it reads.
///
/// - Accounts: the three modules of its pane.
/// - Intelligence: inferd (the model picker and "Your computers"), accountd and syncd (the
///   accounts the local runtimes and the cloud places read).
/// - Spaces: accountd, which keeps the registry.
/// - Assistant Model: inferd, whose picker holds the row.
/// - Privacy: none. The compositor publishes the rows; Settings looks for it and never starts it.
pub fn modules_read(page: &Page) -> &'static [ModuleId] {
    match page {
        Page::Accounts | Page::Intelligence => ACCOUNT_MODULES,
        Page::Spaces => &[ModuleId::Accountd],
        Page::App(_) if is_assistant_model(page) => INFERD,
        Page::Appearance
        | Page::Dock
        | Page::MouseAndGestures
        | Page::KeyboardAndShortcuts
        | Page::Notifications
        | Page::Privacy
        | Page::Power
        | Page::Apps
        | Page::Viewer
        | Page::App(_) => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::modules_read;
    use crate::accounts::ModuleId;
    use ds_settings::schema::Page;

    fn fixed() -> Vec<Page> {
        vec![
            Page::Appearance,
            Page::Dock,
            Page::MouseAndGestures,
            Page::KeyboardAndShortcuts,
            Page::Notifications,
            Page::Spaces,
            Page::Accounts,
            Page::Privacy,
            Page::Power,
            Page::Apps,
            Page::Intelligence,
            Page::Viewer,
            Page::App("acme".to_owned()),
        ]
    }

    #[test]
    fn the_pages_that_read_a_daemon_name_it() {
        assert_eq!(modules_read(&Page::Accounts), ModuleId::ALL);
        assert_eq!(modules_read(&Page::Intelligence), ModuleId::ALL);
        assert_eq!(modules_read(&Page::Spaces), [ModuleId::Accountd]);
        assert_eq!(
            modules_read(&crate::assistant_model::assistant_model_page()),
            [ModuleId::Inferd]
        );
    }

    #[test]
    fn a_page_that_reads_files_only_starts_nothing() {
        for page in [
            Page::Appearance,
            Page::Dock,
            Page::MouseAndGestures,
            Page::KeyboardAndShortcuts,
            Page::Notifications,
            Page::Privacy,
            Page::Power,
            Page::Apps,
            Page::Viewer,
            Page::App("acme".to_owned()),
        ] {
            assert!(modules_read(&page).is_empty(), "{page:?}");
        }
    }

    #[test]
    fn every_daemon_detent_reads_is_named_by_some_page() {
        for module in ModuleId::ALL {
            assert!(
                fixed()
                    .iter()
                    .any(|page| modules_read(page).contains(&module)),
                "{module:?}"
            );
        }
    }

    #[test]
    fn a_page_names_a_daemon_once() {
        for page in fixed() {
            let mut seen = modules_read(&page).to_vec();
            seen.sort();
            seen.dedup();
            assert_eq!(seen.len(), modules_read(&page).len(), "{page:?}");
        }
    }
}
