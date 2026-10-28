//! One page: its blocks, or, when a drill-in row of its advanced keys was pressed, that section's
//! page under quire's page header with its back button.

use crate::advanced_view::{AdvancedPage, Level, level_title};
use crate::env::Env;
use crate::files::{Edit, Files};
use crate::memory_consent::MemoryConsent;
use crate::page_body::PageBody;
use detent_model::{Offers, PageModel, PageSlug, holds_memory_settings};
use dioxus::prelude::*;
use ds::prelude::PageHeader;
use ds_settings::schema::KeyPath;

/// The page `slug`, scrolling; a drill-in row of its advanced keys takes the place of the page
/// until the back button is pressed.
#[component]
pub fn PageView(
    slug: PageSlug,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let env = use_context::<Env>();
    let mut level = use_signal(|| Level::Page);
    // The daemons this page reads are asked to start once, when the page is shown (the window
    // mounts a page afresh for each visit), and never for a page that was not opened.
    let started = env
        .catalogue
        .page(&slug)
        .map_or(&[][..], |page| page.kind.traits().modules);
    use_future({
        let client = env.client.clone();
        move || {
            let client = client.clone();
            async move { client.start(started).await }
        }
    });
    // What waits for the person's word is read each time a page that holds memory settings is
    // shown, since the file may have been edited by hand since.
    let consent = use_context::<MemoryConsent>();
    let holds_memory = env.catalogue.page(&slug).is_some_and(holds_memory_settings);
    use_future(move || {
        let consent = consent.clone();
        async move {
            if holds_memory {
                consent.refresh().await;
                consent.look_at_confinement().await;
            }
        }
    });
    // The choices of the installed Extensions are read once, when a page that has a key choosing
    // among them is shown; coming back to the page after turning one off reads them again.
    let wants_offers = env
        .catalogue
        .page(&slug)
        .is_some_and(PageModel::has_provided);
    let offers = use_hook({
        let extensions = env.extensions.clone();
        move || {
            if wants_offers {
                extensions.read().offers
            } else {
                Offers::default()
            }
        }
    });
    let Some(model) = env.catalogue.page(&slug).map(|page| page.offering(&offers)) else {
        return rsx! {};
    };
    // Each level has its own scroller, so a drill-in and the way back both open at the top (one
    // scroller would keep the long page's offset, past the end of a short section).
    match level() {
        Level::Page => rsx! {
            div { class: "scroll", "data-page": "{slug.0}",
                PageBody {
                    page: model.clone(),
                    hits,
                    files,
                    onedit,
                    on_open: move |name: String| level.set(Level::Advanced(name)),
                }
            }
        },
        Level::Advanced(name) => rsx! {
            div { class: "scroll", "data-page": "{slug.0}", "data-level": "advanced",
                PageHeader {
                    title: level_title(&Level::Advanced(name.clone()), &model),
                    back: Some(model.title.clone()),
                    on_back: move |()| level.set(Level::Page),
                }
                AdvancedPage { page: model.clone(), title: name, hits, files, onedit }
            }
        },
    }
}
