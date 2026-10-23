//! A link from outside to one Extension's row: the page switches and the row is marked.

use detent_model::{EXTENSIONS_SLUG, PageSlug};
use dioxus::prelude::*;

/// The window's means of showing an Extension: provided by the root, used by the page.
#[derive(Clone, Copy)]
pub(crate) struct ExtensionLink {
    page: Signal<PageSlug>,
    query: Signal<String>,
    wanted: Signal<Option<String>>,
}

impl ExtensionLink {
    /// Makes the link over the window's page and search signals; call it once, in the root.
    pub fn provide(page: Signal<PageSlug>, query: Signal<String>) -> ExtensionLink {
        let wanted = use_signal(|| None);
        use_context_provider(|| ExtensionLink {
            page,
            query,
            wanted,
        })
    }

    /// Shows the Extensions page with the row of `id` marked.
    pub fn open(mut self, id: String) {
        self.query.set(String::new());
        self.wanted.set(Some(id));
        self.page.set(PageSlug(EXTENSIONS_SLUG.to_owned()));
    }

    /// The Extension the page is to mark, if a link asked for one.
    pub fn wanted(self) -> Option<String> {
        (self.wanted)()
    }

    /// The page has taken the request: coming back later marks nothing.
    pub fn clear(mut self) {
        self.wanted.set(None);
    }
}
