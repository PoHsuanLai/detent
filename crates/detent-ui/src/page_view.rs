//! One page: its sections, each a card of rows, and the sheet rows of its advanced keys at the end.

use crate::accounts::AccountsPane;
use crate::advanced_view::AdvancedView;
use crate::assistants::Assistants;
use crate::env::Env;
use crate::files::{Edit, Files};
use crate::inference::{LocalRuntimes, ModelPicker, show_reason};
use crate::live::start;
use crate::section_view::SectionView;
use crate::shortcuts::ShortcutsPane;
use crate::spaces::SpacesPane;
use crate::updates::SoftwareUpdatePane;
use detent_model::{Block, PageSlug, blocks, modules_read};
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
    // The daemons this page reads are asked to start once, when the page is shown (the window
    // mounts a page afresh for each visit), and never for a page that was not opened.
    let started = env
        .catalogue
        .page(&slug)
        .map_or(&[][..], |page| modules_read(&page.page));
    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move { start(&bus, started).await }
        }
    });
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
                    Block::AccountsPane => rsx! { AccountsPane {} },
                    Block::SpacesPane => rsx! { SpacesPane {} },
                    Block::Shortcuts => rsx! { ShortcutsPane {} },
                    Block::SoftwareUpdate => rsx! { SoftwareUpdatePane {} },
                    Block::Assistants => rsx! { Assistants {} },
                    Block::LocalRuntimes => rsx! { LocalRuntimes {} },
                    Block::ModelPicker => rsx! {
                        ModelPicker { reasons: show_reason(&env, &files()) }
                    },
                }
            }
            if !page.advanced.is_empty() {
                AdvancedView { slug: slug.clone(), groups: page.advanced.clone(), basic: page.sections.clone(), hits: hits.clone(), files, onedit }
            }
        }
    }
}
