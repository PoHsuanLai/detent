//! One page: its sections, each a card of rows, and the sheet rows of its advanced keys at the end.

use crate::accounts::AccountsPane;
use crate::advanced_view::AdvancedView;
use crate::assistant_model::AssistantModelPane;
use crate::assistants::Assistants;
use crate::bluetooth::BluetoothPane;
use crate::displays::DisplaysPane;
use crate::env::Env;
use crate::extensions::ExtensionsPane;
use crate::files::{Edit, Files};
use crate::hot_corners::HotCorners;
use crate::memory_consent::MemoryConsent;
use crate::picker::{LocalRuntimes, ModelPicker, show_reason};
use crate::privacy::PrivacyPane;
use crate::screen_sharing::ScreenSharingPane;
use crate::section_view::SectionView;
use crate::shortcuts::ShortcutsPane;
use crate::spaces::SpacesPane;
use crate::style_notes::StyleNotesLine;
use crate::updates::SoftwareUpdatePane;
use crate::wifi::WifiPane;
use detent_model::{
    Block, Offers, PageModel, PageSlug, advanced_sections, blocks, holds_memory_settings,
};
use detent_system::bus::start;
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
        .map_or(&[][..], |page| page.kind.modules_read());
    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move { start(&bus, started).await }
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
    let Some(page) = env.catalogue.page(&slug).map(|page| page.offering(&offers)) else {
        return rsx! {};
    };
    let page = &page;
    let advanced = advanced_sections(page);
    rsx! {
        div { class: "scroll", "data-page": "{slug.0}",
            for block in blocks(page) {
                match block {
                    Block::Section(section) => rsx! {
                        SectionView { key: "{section.title}", section, hits: hits.clone(), files, onedit }
                    },
                    Block::AccountsPane => rsx! { AccountsPane {} },
                    Block::PrivacyPane => rsx! { PrivacyPane {} },
                    Block::ScreenSharing => rsx! { ScreenSharingPane {} },
                    Block::SpacesPane => rsx! { SpacesPane {} },
                    Block::Shortcuts => rsx! { ShortcutsPane {} },
                    Block::SoftwareUpdate => rsx! { SoftwareUpdatePane {} },
                    Block::AssistantModel => rsx! { AssistantModelPane {} },
                    Block::Wifi => rsx! { WifiPane {} },
                    Block::Bluetooth => rsx! { BluetoothPane {} },
                    Block::Displays => rsx! { DisplaysPane {} },
                    Block::Extensions => rsx! { ExtensionsPane {} },
                    Block::Assistants => rsx! { Assistants {} },
                    Block::LocalRuntimes => rsx! { LocalRuntimes {} },
                    Block::StyleNotes => rsx! { StyleNotesLine {} },
                    Block::HotCorners(entries) => rsx! { HotCorners { entries, files, onedit } },
                    Block::ModelPicker => rsx! {
                        ModelPicker { reasons: show_reason(&env, &files()) }
                    },
                }
            }
            if !advanced.is_empty() {
                AdvancedView { slug: slug.clone(), groups: advanced, basic: page.sections.clone(), hits: hits.clone(), files, onedit }
            }
        }
    }
}
