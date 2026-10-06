//! One page: its sections, each a card of rows, and one "Advanced" disclosure at the end.

use crate::advanced_view::AdvancedView;
use crate::env::Env;
use crate::files::{Edit, Files};
use crate::inference::{ModelPicker, show_reason};
use crate::section_view::SectionView;
use detent_model::{Block, PageSlug, blocks};
use dioxus::prelude::*;
use ds_settings::schema::KeyPath;

/// The sections of the page `slug`, then its advanced keys, scrolling.
#[component]
pub fn PageView(
    slug: PageSlug,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let env = use_context::<Env>();
    let Some(page) = env.catalogue.page(&slug) else {
        return rsx! {};
    };
    rsx! {
        div { class: "scroll", "data-page": "{slug.0}",
            for block in blocks(page) {
                match block {
                    Block::Section(section) => rsx! {
                        SectionView { key: "{section.title}", section, hits: hits.clone(), files, onedit }
                    },
                    Block::ModelPicker => rsx! {
                        ModelPicker { reasons: show_reason(&env, &files()) }
                    },
                }
            }
            if !page.advanced.is_empty() {
                AdvancedView { slug: slug.clone(), groups: page.advanced.clone(), hits: hits.clone(), files, onedit }
            }
        }
    }
}
