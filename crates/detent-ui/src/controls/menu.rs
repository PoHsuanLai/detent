//! More than five words: a pop-up button.

use detent_model::humanize;
use dioxus::prelude::*;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// A pop-up button over the words, the stored word chosen.
#[component]
pub fn MenuControl(
    words: Vec<String>,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let current = value.as_str().map(str::to_owned);
    let items: Vec<MenuItem<String>> = words
        .iter()
        .map(|word| MenuItem::new(word.clone(), humanize(word)))
        .collect();
    rsx! {
        PopUpButton::<String> {
            items,
            value: current,
            size: ControlSize::Small,
            onpick: move |word| onchange.call(toml::Value::String(word)),
        }
    }
}
