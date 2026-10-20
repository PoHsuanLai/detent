//! The row a system page shows when there is nothing to show.

use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;

/// A card with one disabled row: the page's name, and why there is nothing on it.
#[component]
pub(crate) fn Unavailable(title: String, words: String) -> Element {
    rsx! {
        div { class: "system-page", "data-system": "unavailable",
            FieldGroup {
                FieldRow {
                    label: TextLine::from(title),
                    help: Some(TextLine::from(words)),
                    availability: Availability::Disabled,
                    Label { text: "Unavailable", role: LabelRole::Secondary }
                }
            }
        }
    }
}
