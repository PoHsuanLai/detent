//! Which pages the sidebar lists and which one the window shows.

use crate::catalogue::{Catalogue, PageModel};
use crate::page::PageSlug;
use crate::search::Search;

/// The pages the sidebar lists: all of them while the search box is empty, otherwise the ones
/// with a hit.
pub fn visible_pages<'a>(catalogue: &'a Catalogue, search: &Search) -> Vec<&'a PageModel> {
    match search {
        Search::Idle => catalogue.pages.iter().collect(),
        Search::Results(hits) => catalogue
            .pages
            .iter()
            .filter(|page| hits.iter().any(|hit| hit.page == page.slug))
            .collect(),
    }
}

/// The page to show when `wanted` is asked for: `wanted` if it is listed, else the page of the
/// best hit, else the first listed page.
pub fn landing<'a>(
    visible: &[&'a PageModel],
    search: &Search,
    wanted: &PageSlug,
) -> Option<&'a PageModel> {
    let listed = |slug: &PageSlug| visible.iter().find(|page| &page.slug == slug).copied();
    listed(wanted)
        .or_else(|| search.first().and_then(|hit| listed(&hit.page)))
        .or_else(|| visible.first().copied())
}
