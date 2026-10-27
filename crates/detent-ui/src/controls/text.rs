//! Free text: a text field that stores when the caret leaves or Enter is pressed.

use detent_model::external::toml;
use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// A text field holding a draft while it is edited.
#[component]
pub fn TextControl(
    label: String,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let stored = value.as_str().unwrap_or_default().to_owned();
    let mut draft = use_signal(|| stored.clone());
    rsx! {
        TextField {
            label,
            value: draft(),
            size: ControlSize::Small,
            oninput: move |next| draft.set(next),
            onchange: move |next: String| onchange.call(toml::Value::String(next)),
        }
    }
}
