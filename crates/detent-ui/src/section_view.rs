//! One section: a titled card of its rows, and the grey footnote under it when the schema has one.

use crate::files::{Edit, Files};
use crate::key_row::{KeyBlock, KeyRow};
use crate::memory_consent::ConfinementNote;
use detent_model::{Control as Kind, KeyEntry, SectionModel, tells_confinement};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::FieldGroup;
use ds::prelude::Label;
use ds_settings::schema::KeyPath;

/// Whether a search found `entry`.
pub(crate) fn is_hit(entry: &KeyEntry, hits: &[KeyPath]) -> bool {
    hits.contains(&entry.spec.path)
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
    let footnote = section.footnote.clone();
    let (blocks, rows): (Vec<KeyEntry>, Vec<KeyEntry>) =
        section.keys.iter().cloned().partition(is_block);
    let footnote = footnote.filter(|_| !rows.is_empty());
    rsx! {
        div { class: "section",
            if !rows.is_empty() {
                FieldGroup { title,
                    for entry in rows {
                        KeyRow { key: "{entry.spec.path.0}", hit: is_hit(&entry, &hits), entry, files, onedit }
                    }
                }
            }
            if let Some(words) = footnote {
                div { class: "section-footnote",
                    Label { text: words, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
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
