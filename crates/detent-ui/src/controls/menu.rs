//! More than five words: a pop-up button.

use crate::copy::NOT_INSTALLED;
use detent_model::{stray_word, word_label};
use dioxus::prelude::*;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::WordLabels;

/// A pop-up button over the words, the stored word chosen. A stored word that is not among them
/// (a look whose Extension was removed or turned off) is kept: it is listed greyed, with why under
/// it, and stays the one shown until another is picked.
#[component]
pub fn MenuControl(
    words: Vec<String>,
    labels: WordLabels,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let current = value.as_str().map(str::to_owned);
    let gone = stray_word(&words, &value).map(|word| {
        [
            MenuItem::new(word.to_owned(), word.to_owned())
                .with_availability(Availability::Disabled),
            MenuItem::Info {
                title: NOT_INSTALLED.to_owned(),
                detail: None,
            },
        ]
    });
    let items: Vec<MenuItem<String>> = words
        .iter()
        .map(|word| MenuItem::new(word.clone(), word_label(&labels, word)))
        .chain(gone.into_iter().flatten())
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
