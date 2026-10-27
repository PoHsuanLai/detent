//! The sidebar's page rows.

use crate::icons::icon_of;
use detent_model::{Catalogue, PageSlug};
use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};

/// One row per listed page, `current` drawn selected; a click sets `page`.
pub fn sidebar_items(
    catalogue: &Catalogue,
    listed: &[PageSlug],
    current: Option<&PageSlug>,
    mut page: Signal<PageSlug>,
) -> Vec<ListItem<String>> {
    listed
        .iter()
        .filter_map(|slug| catalogue.page(slug))
        .map(|model| {
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
                        leading: RowLeading::Icon(icon_of(model.kind)),
                        state: RowState { selection, ..RowState::default() },
                        common: Common {
                            data: DataName::parse("page-row")
                                .map(|name| vec![DataAttr::new(name, slug.0.clone())])
                                .unwrap_or_default(),
                            ..Common::default()
                        },
                        onclick: move |_| page.set(slug.clone()),
                    }
                },
            )
        })
        .collect()
}
