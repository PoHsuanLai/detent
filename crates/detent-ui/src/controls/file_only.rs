//! A kind quire has no component for: its value, read-only, and where to change it.

use detent_model::external::toml;
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::prelude::*;

/// The value as one line of text: a list's items joined, anything else as it is written.
fn line_of(value: &toml::Value) -> String {
    match value {
        toml::Value::String(text) => text.clone(),
        toml::Value::Array(items) => items.iter().map(line_of).collect::<Vec<_>>().join(", "),
        other => other.to_string(),
    }
}

/// The value with an "Edit in file" note under it.
#[component]
pub fn FileOnlyControl(value: toml::Value) -> Element {
    rsx! {
        div { class: "key-file-only",
            Label { text: line_of(&value), role: LabelRole::Secondary }
            Label { text: "Edit in file", role: LabelRole::Tertiary, style: LabelStyle::Footnote }
        }
    }
}
