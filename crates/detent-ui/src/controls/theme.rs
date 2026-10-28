//! The theme: three choices, each a small picture of a window in that palette, side by side.

use super::theme_picture::{SIDE, picture_of};
use crate::marked::marked;
use detent_model::external::toml;
use detent_model::word_label;
use dioxus::prelude::*;
use ds::components::content::icon_source::{ExternalIcon, IconSource};
use ds::components::controls::radio_group::Arrangement;
use ds::prelude::*;
use ds::style::icon::render::{IconPx, IconSize};
use ds_settings::schema::WordLabels;

/// The order the choices are drawn in: light, dark, then the one that follows the desktop.
const ORDER: [&str; 3] = ["light", "dark", "system"];

/// Whether the words are exactly the theme's three: such an enum is drawn as pictures.
pub fn theme_words(words: &[String]) -> bool {
    words.len() == ORDER.len()
        && ORDER
            .iter()
            .all(|word| words.iter().any(|have| have == word))
}

/// One choice for `word`: its label over its picture.
fn choice_of(word: &str, labels: &WordLabels) -> Choice<String> {
    let choice = Choice::new(word.to_owned(), word_label(labels, word));
    match picture_of(word) {
        Some(url) => choice.with_icon(IconSource::Image(ExternalIcon {
            url,
            size: IconSize::Px(IconPx(SIDE)),
        })),
        None => choice,
    }
}

/// The three theme choices with the stored word checked; a pick is sent as the word to store.
/// `path` marks the group (`data-key`) for the window's tests.
#[component]
pub fn ThemeControl(
    label: String,
    path: String,
    labels: WordLabels,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let current = value.as_str().unwrap_or_default().to_owned();
    let choices: Vec<Choice<String>> = ORDER.iter().map(|word| choice_of(word, &labels)).collect();
    rsx! {
        RadioGroup::<String> {
            label,
            choices,
            value: current,
            arrangement: Arrangement::Row,
            common: marked("key", &path),
            onchange: move |word: String| onchange.call(toml::Value::String(word)),
        }
    }
}
