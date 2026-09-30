//! One section: a titled card of its basic keys, and the advanced ones under a disclosure at
//! its end.

use crate::files::{Edit, Files};
use crate::key_row::{KeyRow, Match};
use detent_model::{KeyEntry, SectionModel};
use dioxus::prelude::*;
use ds::components::controls::disclosure::Disclosure;
use ds::components::fields::field_row::FieldGroup;
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::KeyPath;

fn is_hit(entry: &KeyEntry, hits: &[KeyPath]) -> bool {
    hits.contains(&entry.spec.path)
}

fn match_of(entry: &KeyEntry, hits: &[KeyPath]) -> Match {
    if is_hit(entry, hits) {
        Match::Hit
    } else {
        Match::Miss
    }
}

/// A name for the section's disclosure that no other on the page has: its first advanced key's
/// path.
fn advanced_id(section: &SectionModel) -> String {
    section
        .advanced
        .first()
        .map(|entry| entry.spec.path.0.replace('.', "-"))
        .unwrap_or_default()
}

/// A section's card.
#[component]
pub fn SectionView(
    section: SectionModel,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let mut opened = use_signal(|| Shown::Hidden);
    // A search hit inside the disclosure opens it, so the highlighted row is on screen.
    let shown = if section.advanced.iter().any(|entry| is_hit(entry, &hits)) {
        Shown::Visible
    } else {
        opened()
    };
    let title = (!section.title.is_empty()).then(|| section.title.clone());
    rsx! {
        div { class: "section",
            if section.basic.is_empty() {
                if let Some(title) = title.clone() {
                    SectionHeader { title }
                }
            } else {
                FieldGroup { title,
                    for entry in section.basic.iter().cloned() {
                        KeyRow { key: "{entry.spec.path.0}", found: match_of(&entry, &hits), entry, files, onedit }
                    }
                }
            }
            if !section.advanced.is_empty() {
                div { class: "advanced",
                    Disclosure {
                        common: Common { id: Some(format!("advanced-{}", advanced_id(&section))), ..Common::default() },
                        label: Some("Advanced".to_owned()),
                        shown,
                        on_toggle: move |next| opened.set(next),
                        FieldGroup {
                            for entry in section.advanced.iter().cloned() {
                                KeyRow { key: "{entry.spec.path.0}", found: match_of(&entry, &hits), entry, files, onedit }
                            }
                        }
                    }
                }
            }
        }
    }
}
