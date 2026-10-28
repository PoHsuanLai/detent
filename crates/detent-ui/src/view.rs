//! What the window shows for a search box and a chosen page, worked out once per render.

use detent_model::{Catalogue, Hit, PageSlug, landing, search, visible_pages};

/// The sidebar's pages, the page shown and the keys the search found.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    /// The page shown, `None` when the catalogue is empty or nothing matches.
    pub current: Option<PageSlug>,
    /// The pages the sidebar lists, in order.
    pub listed: Vec<PageSlug>,
    /// The keys the search found, best first.
    pub hits: Vec<Hit>,
    /// Whether a query is in the box.
    pub searching: bool,
}

impl View {
    /// The view of `catalogue` for `query`, with `wanted` the page the person chose last.
    pub fn of(catalogue: &Catalogue, query: &str, wanted: &PageSlug) -> View {
        let found = search(catalogue, query);
        let visible = visible_pages(catalogue, &found);
        let current = landing(&visible, &found, wanted).map(|page| page.slug.clone());
        View {
            current,
            listed: visible.iter().map(|page| page.slug.clone()).collect(),
            hits: found.hits().to_vec(),
            searching: !query.trim().is_empty(),
        }
    }
}
