//! A page's advanced keys: one drill-in row per section; a press shows a page of that section's
//! rows in place of the page, under quire's `PageHeader` with its back button.

use crate::files::{Edit, Files};
use crate::icons::{GREY, tile_on};
use crate::key_row::HIT_CLASS;
use crate::marked::marked;
use crate::section_view::{SectionView, is_hit};
use detent_model::{PageModel, PageSlug, SectionModel, advanced_name};
use dioxus::prelude::*;
use ds::components::fields::field_row::FieldGroup;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;
use ds_settings::schema::KeyPath;

/// What a page shows: itself, or one of its advanced sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Level {
    /// The page itself.
    Page,
    /// The advanced section titled so.
    Advanced(String),
}

/// The name of `level` on `page`: the page's own title, or the section's name beside the page's
/// basic sections.
pub(crate) fn level_title(level: &Level, page: &PageModel) -> String {
    match level {
        Level::Page => page.title.clone(),
        Level::Advanced(title) => page
            .advanced
            .iter()
            .find(|group| &group.title == title)
            .map_or_else(
                || title.clone(),
                |group| advanced_name(group, &page.sections),
            ),
    }
}

/// The advanced `groups` of the page `slug`, one drill-in row each in a card of their own, named
/// by [`advanced_name`] against the page's `basic` sections. A search hit inside a section
/// highlights its row; a press hands the section's title to `on_open`.
#[component]
pub(crate) fn AdvancedView(
    slug: PageSlug,
    groups: Vec<SectionModel>,
    basic: Vec<SectionModel>,
    hits: Vec<KeyPath>,
    on_open: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "advanced", "data-page-advanced": "{slug.0}",
            FieldGroup {
                for group in groups.iter() {
                    DrillRow {
                        key: "{group.title}",
                        name: advanced_name(group, &basic),
                        title: group.title.clone(),
                        hit: group.keys.iter().any(|entry| is_hit(entry, &hits)),
                        on_open,
                    }
                }
            }
        }
    }
}

/// One section's row: a grey tile, its name and a chevron.
#[component]
fn DrillRow(name: String, title: String, hit: bool, on_open: EventHandler<String>) -> Element {
    let class = hit.then(|| ExtraClass::parse(HIT_CLASS).ok()).flatten();
    let common = Common {
        extra_class: class,
        ..marked("advanced-row", &title)
    };
    rsx! {
        Row {
            leading: RowLeading::Tile(tile_on(Icon::Switches, GREY)),
            title: TextLine::from(name),
            accessory: Accessory::Chevron,
            size: RowSize::Settings,
            common,
            onclick: move |_| on_open.call(title.clone()),
        }
    }
}

/// The page of the advanced section `title` of `page`: its rows, under the stack's own
/// header, which the page view draws.
#[component]
pub(crate) fn AdvancedPage(
    page: PageModel,
    title: String,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let section = page
        .advanced
        .iter()
        .find(|group| group.title == title)
        .map(|group| SectionModel {
            title: String::new(),
            footnote: group.footnote.clone(),
            keys: group.keys.clone(),
        });
    rsx! {
        if let Some(section) = section {
            div { class: "advanced-page", "data-advanced-page": "{title}",
                SectionView { section, hits, files, onedit }
            }
        }
    }
}
