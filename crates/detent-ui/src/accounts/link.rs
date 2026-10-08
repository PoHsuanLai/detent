//! A link from another page to one account in the Accounts pane: the page switches and the pane
//! opens that account when it has read its rows.

use detent_model::{AccountKey, AccountTarget, AccountsInput, AccountsPane, PageSlug};
use dioxus::prelude::*;
use ds_settings::schema::Page;

/// What the pane is asked to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Wanted {
    /// An account the window already knows by key.
    Key(AccountKey),
    /// A request from outside: an account's segment, or the list.
    Target(AccountTarget),
}

impl Wanted {
    /// The pane's input: an account unknown to the pane gives the list.
    pub fn input(self, pane: &AccountsPane) -> AccountsInput {
        let key = match self {
            Wanted::Key(key) => Some(key),
            Wanted::Target(AccountTarget::List) => None,
            Wanted::Target(AccountTarget::Detail(segment)) => pane.key_of(&segment),
        };
        key.map_or(AccountsInput::Back, AccountsInput::Open)
    }
}

/// The window's means of showing an account: provided by the root, used by any page that lists
/// accounts (the Intelligence page's local runtimes) and by the pane itself.
#[derive(Clone, Copy)]
pub(crate) struct AccountLink {
    page: Signal<PageSlug>,
    query: Signal<String>,
    wanted: Signal<Option<Wanted>>,
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
    pub fn open(self, key: AccountKey) {
        self.show(Wanted::Key(key));
    }

    /// Shows the Accounts page as an outside request asks.
    pub fn open_target(self, target: AccountTarget) {
        self.show(Wanted::Target(target));
    }

    fn show(mut self, wanted: Wanted) {
        self.query.set(String::new());
        self.wanted.set(Some(wanted));
        self.page.set(PageSlug::of(&Page::Accounts));
    }

    /// The account the pane is to open, if a link asked for one.
    pub fn wanted(self) -> Option<Wanted> {
        (self.wanted)()
    }

    /// The pane has dealt with the request.
    pub fn clear(mut self) {
        self.wanted.set(None);
    }
}
