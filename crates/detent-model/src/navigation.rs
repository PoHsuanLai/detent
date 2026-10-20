//! Which pages the sidebar lists and which one the window shows.

use crate::catalogue::{Catalogue, PageModel};
use crate::page::PageSlug;
use crate::search::Search;

/// The pages the sidebar lists: all of them while the search box is empty, otherwise the ones
/// with a hit or a name the query spells.
pub fn visible_pages<'a>(catalogue: &'a Catalogue, search: &Search) -> Vec<&'a PageModel> {
    match search {
        Search::Idle => catalogue.pages.iter().collect(),
        Search::Results(found) => catalogue
            .pages
            .iter()
            .filter(|page| {
                found.hits.iter().any(|hit| hit.page == page.slug)
                    || found.pages.contains(&page.slug)
            })
            .collect(),
    }
}

/// The page to show when `wanted` is asked for: `wanted` if it is listed, else
/// the first page the query names, else the best hit's, else the first listed page.
pub fn landing<'a>(
    visible: &[&'a PageModel],
    search: &Search,
    wanted: &PageSlug,
) -> Option<&'a PageModel> {
    let listed = |slug: &PageSlug| visible.iter().find(|page| &page.slug == slug).copied();
    listed(wanted)
        .or_else(|| search.first_page().and_then(listed))
        .or_else(|| visible.first().copied())
}

/// The page the window opens on when nothing asks for another: Appearance, the first page of how
/// the desktop looks, else the first page listed.
pub fn opening_page(catalogue: &Catalogue) -> Option<PageSlug> {
    let appearance = PageSlug::of(&ds_settings::schema::Page::Appearance);
    catalogue
        .page(&appearance)
        .or_else(|| catalogue.pages.first())
        .map(|page| page.slug.clone())
}
