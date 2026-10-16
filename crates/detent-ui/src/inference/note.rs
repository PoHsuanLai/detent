//! The module's short line under a row, in plain secondary text.

use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::prelude::*;

/// A note under a row; `kind` tells the places it appears in apart (`data-note`).
#[component]
pub(super) fn NoteLine(text: String, kind: &'static str) -> Element {
    rsx! {
        div { class: "picker-auto", "data-note": kind,
            Label { text, role: LabelRole::Secondary, style: LabelStyle::Footnote }
        }
    }
}
