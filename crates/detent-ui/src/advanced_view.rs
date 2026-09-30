//! The page's one "Advanced" disclosure: every advanced key of the page, grouped by section.

use crate::files::{Edit, Files};
use crate::key_row::KeyRow;
use crate::section_view::{is_hit, match_of};
use detent_model::{PageSlug, SectionModel};
use dioxus::prelude::*;
use ds::components::controls::disclosure::Disclosure;
use ds::components::fields::field_row::FieldGroup;
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::KeyPath;

/// The disclosure's id: the page's slug, made safe for a selector (`app:x` has a colon).
fn disclosure_id(slug: &PageSlug) -> String {
    let safe: String = slug
        .0
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("advanced-{safe}")
}

/// The advanced `groups` of the page `slug`, closed until opened; a search hit among them opens
/// it, so the highlighted row is on screen.
#[component]
pub fn AdvancedView(
    slug: PageSlug,
    groups: Vec<SectionModel>,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let mut opened = use_signal(|| Shown::Hidden);
    let searched = groups
        .iter()
        .flat_map(|group| &group.keys)
        .any(|entry| is_hit(entry, &hits));
    let shown = if searched { Shown::Visible } else { opened() };
    rsx! {
        div { class: "advanced",
            Disclosure {
                common: Common { id: Some(disclosure_id(&slug)), ..Common::default() },
                label: Some("Advanced".to_owned()),
                shown,
                on_toggle: move |next| opened.set(next),
                for group in groups.iter().cloned() {
                    FieldGroup { key: "{group.title}", title: (!group.title.is_empty()).then(|| group.title.clone()),
                        for entry in group.keys.iter().cloned() {
                            KeyRow { key: "{entry.spec.path.0}", found: match_of(&entry, &hits), entry, files, onedit }
                        }
                    }
                }
            }
        }
    }
}
