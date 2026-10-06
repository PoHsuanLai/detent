//! A link from another page to one account in the Accounts pane: the page switches and the pane
//! opens that account when it has read its rows.

use detent_model::{AccountKey, PageSlug};
use dioxus::prelude::*;
use ds_settings::schema::Page;

/// The window's means of showing an account: provided by the root, used by any page that lists
/// accounts (the Intelligence page's local runtimes) and by the pane itself.
#[derive(Clone, Copy)]
pub(crate) struct AccountLink {
    page: Signal<PageSlug>,
    query: Signal<String>,
    wanted: Signal<Option<AccountKey>>,
}

impl AccountLink {
    /// Makes the link over the window's page and search signals; call it once, in the root.
    pub fn provide(page: Signal<PageSlug>, query: Signal<String>) -> AccountLink {
        let wanted = use_signal(|| None);
        use_context_provider(|| AccountLink {
            page,
            query,
            wanted,
        })
    }

    /// Shows the Accounts page with `key` opened.
    pub fn open(mut self, key: AccountKey) {
        self.query.set(String::new());
        self.wanted.set(Some(key));
        self.page.set(PageSlug::of(&Page::Accounts));
    }

    /// The account the pane is to open, if a link asked for one.
    pub fn wanted(self) -> Option<AccountKey> {
        (self.wanted)()
    }

    /// The pane has dealt with the request.
    pub fn clear(mut self) {
        self.wanted.set(None);
    }
}
