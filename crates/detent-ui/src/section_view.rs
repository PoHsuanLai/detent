//! One section: a titled card of its rows.

use crate::files::{Edit, Files};
use crate::key_row::{KeyBlock, KeyRow, Match};
use crate::memory_consent::ConfinementNote;
use detent_model::{Control as Kind, KeyEntry, SectionModel, tells_confinement};
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

/// Whether `entry` is drawn as a block of its own and not as a row of the card (the pictures).
fn is_block(entry: &KeyEntry) -> bool {
    matches!(entry.control, Kind::Picture)
}

/// A section's card: its title, when it has one, over its rows. The Memory section ends with the
/// line about memory's confinement. A key that is a block of its own (the pictures of the
/// Wallpaper page) stands under the card, which is left out when it has no row.
#[component]
pub fn SectionView(
    section: SectionModel,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let title = (!section.title.is_empty()).then(|| section.title.clone());
    let confinement = tells_confinement(&section);
    let (blocks, rows): (Vec<KeyEntry>, Vec<KeyEntry>) =
        section.keys.iter().cloned().partition(is_block);
    rsx! {
        div { class: "section",
            if !rows.is_empty() {
                FieldGroup { title,
                    for entry in rows {
                        KeyRow { key: "{entry.spec.path.0}", found: match_of(&entry, &hits), entry, files, onedit }
                    }
                }
            }
            for entry in blocks {
                KeyBlock { key: "{entry.spec.path.0}", entry, files, onedit }
            }
            if confinement {
                ConfinementNote {}
            }
        }
    }
}
