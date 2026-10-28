//! One page's content: its hero when it has one, its blocks in order, then the drill-in rows of
//! its advanced keys.

use crate::accounts::AccountsPane;
use crate::advanced_view::AdvancedView;
use crate::assistant_model::AssistantModelPane;
use crate::assistants::Assistants;
use crate::bluetooth::BluetoothPane;
use crate::displays::DisplaysPane;
use crate::env::Env;
use crate::extensions::ExtensionsPane;
use crate::files::{Edit, Files};
use crate::hero_view::HeroView;
use crate::hot_corners::HotCorners;
use crate::picker::{LocalRuntimes, ModelPicker, show_reason};
use crate::privacy::PrivacyPane;
use crate::screen_sharing::ScreenSharingPane;
use crate::section_view::SectionView;
use crate::shortcuts::ShortcutsPane;
use crate::spaces::SpacesPane;
use crate::style_notes::StyleNotesLine;
use crate::updates::SoftwareUpdatePane;
use crate::wifi::WifiPane;
use detent_model::{Block, LivePane, PageModel, advanced_sections, blocks};
use dioxus::prelude::*;
use ds_settings::schema::KeyPath;

/// The blocks of `page`, then a drill-in row for each advanced section; a press on a row hands
/// the section's title to `on_open`.
#[component]
pub(crate) fn PageBody(
    page: PageModel,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
    on_open: EventHandler<String>,
) -> Element {
    let env = use_context::<Env>();
    let advanced = advanced_sections(&page);
    rsx! {
        if page.kind.traits().hero {
            HeroView { kind: page.kind, title: page.title.clone() }
        }
        for block in blocks(&page) {
            match block {
                Block::Section(section) => rsx! {
                    SectionView { key: "{section.title}", section, hits: hits.clone(), files, onedit }
                },
                Block::Pane(pane) => mount(pane, &env, files),
                Block::StyleNotes => rsx! { StyleNotesLine {} },
                Block::HotCorners(entries) => rsx! { HotCorners { entries, files, onedit } },
            }
        }
        if !advanced.is_empty() {
            AdvancedView {
                slug: page.slug.clone(),
                groups: advanced,
                basic: page.sections.clone(),
                hits: hits.clone(),
                on_open,
            }
        }
    }
}

/// The pane that fills `pane`'s rows.
fn mount(pane: LivePane, env: &Env, files: Signal<Files>) -> Element {
    match pane {
        LivePane::Accounts => rsx! { AccountsPane {} },
        LivePane::Privacy => rsx! { PrivacyPane {} },
        LivePane::ScreenSharing => rsx! { ScreenSharingPane {} },
        LivePane::Spaces => rsx! { SpacesPane {} },
        LivePane::Shortcuts => rsx! { ShortcutsPane {} },
        LivePane::SoftwareUpdate => rsx! { SoftwareUpdatePane {} },
        LivePane::AssistantModel => rsx! { AssistantModelPane {} },
        LivePane::Wifi => rsx! { WifiPane {} },
        LivePane::Bluetooth => rsx! { BluetoothPane {} },
        LivePane::Displays => rsx! { DisplaysPane {} },
        LivePane::Extensions => rsx! { ExtensionsPane {} },
        LivePane::Assistants => rsx! { Assistants {} },
        LivePane::LocalRuntimes => rsx! { LocalRuntimes {} },
        LivePane::ModelPicker => rsx! { ModelPicker { reasons: show_reason(env, &files()) } },
    }
}
