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
use detent_model::{Block, Head, PageModel, advanced_sections, blocks};
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
        if let Head::Hero(hero) = page.kind.head() {
            HeroView { kind: page.kind, hero, title: page.title.clone() }
        }
        for block in blocks(&page) {
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
