//! The control each key kind gets (design/22 section 9.1's fixed table): one component per
//! [`detent_model::Control`], all of them quire's.

mod file_only;
mod menu;
mod segmented;
mod slider;
mod swatches;
mod text;
mod toggle;

use detent_model::{Control as Kind, KeyEntry};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::prelude::*;
pub(crate) use file_only::FileOnlyControl;
use menu::MenuControl;
use segmented::SegmentedKey;
use slider::SliderControl;
use swatches::{SwatchControl, swatch_words};
use text::TextControl;
use toggle::ToggleControl;

/// The control for `entry`, showing `value`; a change is sent to `onchange` as the value to
/// store.
#[component]
pub fn Control(
    entry: KeyEntry,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let label = entry.spec.label.0.clone();
    let labels = entry.spec.labels.clone();
    match entry.control {
        Kind::Toggle(pair) => rsx! { ToggleControl { label, pair, value, onchange } },
        Kind::Segmented(words) | Kind::Menu(words) if swatch_words(&words) => {
            rsx! { SwatchControl { label, value, onchange } }
        }
        Kind::Segmented(words) => rsx! { SegmentedKey { label, words, labels, value, onchange } },
        Kind::Menu(words) => rsx! { MenuControl { words, labels, value, onchange } },
        Kind::Slider(range) => rsx! { SliderControl { label, range, value, onchange } },
        Kind::Text => rsx! { TextControl { label, value, onchange } },
        Kind::Colour => rsx! { SwatchControl { label, value, onchange } },
        Kind::Readout(word) => {
            rsx! { Label { text: detent_model::word_label(&labels, &word), role: LabelRole::Secondary } }
        }
        Kind::FileOnly(_) => rsx! { FileOnlyControl { value } },
    }
}
