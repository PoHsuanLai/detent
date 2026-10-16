//! One section: a titled card of its rows.

use crate::files::{Edit, Files};
use crate::key_row::{KeyRow, Match};
use crate::memory::ConfinementNote;
use detent_model::{KeyEntry, SectionModel, tells_confinement};
use dioxus::prelude::*;
use ds::components::fields::field_row::FieldGroup;
use ds_settings::schema::KeyPath;

/// Whether a search found `entry`.
pub(crate) fn is_hit(entry: &KeyEntry, hits: &[KeyPath]) -> bool {
    hits.contains(&entry.spec.path)
}

/// `entry`'s row highlight.
pub(crate) fn match_of(entry: &KeyEntry, hits: &[KeyPath]) -> Match {
    if is_hit(entry, hits) {
        Match::Hit
    } else {
        Match::Miss
    }
}

/// A section's card: its title, when it has one, over its rows. The Memory section ends with the
/// line about memory's confinement.
#[component]
pub fn SectionView(
    section: SectionModel,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let title = (!section.title.is_empty()).then(|| section.title.clone());
    let confinement = tells_confinement(&section);
    rsx! {
        div { class: "section",
            FieldGroup { title,
                for entry in section.keys.iter().cloned() {
                    KeyRow { key: "{entry.spec.path.0}", found: match_of(&entry, &hits), entry, files, onedit }
                }
            }
            if confinement {
                ConfinementNote {}
            }
        }
    }
}
