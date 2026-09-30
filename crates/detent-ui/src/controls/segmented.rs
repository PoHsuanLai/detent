//! Three to five words: a segmented control.

use detent_model::word_label;
use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::WordLabels;

/// One segment per word, the stored word selected.
#[component]
pub fn SegmentedKey(
    label: String,
    words: Vec<String>,
    labels: WordLabels,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let current = value.as_str().unwrap_or_default().to_owned();
    rsx! {
        SegmentedControl::<String> {
            label,
            choices: Choice::pairs(words.iter().map(|word| (word.clone(), word_label(&labels, word)))),
            tracking: Tracking::SelectOne(current),
            size: ControlSize::Small,
            onchange: move |word| onchange.call(toml::Value::String(word)),
        }
    }
}
