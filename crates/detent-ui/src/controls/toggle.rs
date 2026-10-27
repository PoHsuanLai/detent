//! A two-word enum: a switch.

use detent_model::TogglePair;
use detent_model::external::toml;
use dioxus::prelude::*;
use ds::prelude::*;

/// On when the value is the pair's on word.
#[component]
pub fn ToggleControl(
    label: String,
    pair: TogglePair,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let on = value.as_str() == Some(pair.on.as_str());
    rsx! {
        Toggle {
            label,
            value: if on { Check::On } else { Check::Off },
            onchange: move |next| {
                let word = match next {
                    Check::On => pair.on.clone(),
                    Check::Off | Check::Mixed => pair.off.clone(),
                };
                onchange.call(toml::Value::String(word));
            },
        }
    }
}
