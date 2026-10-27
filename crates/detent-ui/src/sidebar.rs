//! The sidebar's page rows, in groups with a gap between them.

use crate::icons::tile_of;
use crate::marked::marked;
use detent_model::{Catalogue, PageModel, PageSlug, sidebar_groups};
use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::prelude::*;

/// The class of the empty heading that makes the gap between two groups.
pub(crate) const GAP_CLASS: &str = "sidebar-gap";

/// One row per listed page, grouped by [`sidebar_groups`] with a gap item between groups,
/// `current` drawn selected; a click sets `page`.
pub fn sidebar_items(
    catalogue: &Catalogue,
    listed: &[PageSlug],
    current: Option<&PageSlug>,
    page: Signal<PageSlug>,
) -> Vec<ListItem<String>> {
    let models: Vec<&PageModel> = listed
        .iter()
        .filter_map(|slug| catalogue.page(slug))
        .collect();
    sidebar_groups(&models)
        .into_iter()
        .enumerate()
        .flat_map(|(at, group)| {
            let gap = (at > 0)
                .then(|| ListItem::heading(format!("gap-{at}"), rsx! { div { class: GAP_CLASS } }));
            let rows = group
                .into_iter()
                .map(move |model| page_row(model, current, page));
            gap.into_iter().chain(rows)
        })
        .collect()
}

/// The row of `model`, selected when it is `current`.
fn page_row(
    model: &PageModel,
    current: Option<&PageSlug>,
    mut page: Signal<PageSlug>,
) -> ListItem<String> {
    let slug = model.slug.clone();
    let label = model.title.clone();
    let selection = if current == Some(&slug) {
        Selection::Selected
    } else {
        Selection::Unselected
    };
    ListItem::row(
        slug.0.clone(),
        label.clone(),
        rsx! {
            Row {
                title: TextLine::from(label),
                leading: RowLeading::Tile(tile_of(model.kind)),
                state: RowState { selection, ..RowState::default() },
                common: marked("page-row", &slug.0),
                onclick: move |_| page.set(slug.clone()),
            }
        },
    )
}
