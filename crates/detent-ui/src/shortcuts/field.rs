//! How a chord is shown and changed in a row: the one place that names the recorder. This file
//! shows the chord and clears it; `field_recorder.rs` is the same component over quire's
//! recorder (`ShortcutField`), and replaces this file when quire has it.

use detent_model::{ShortcutRow, ShortcutsInput};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::ButtonRole;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// What a row without a chord says.
const NO_CHORD: &str = "None";

/// The chord of `row`, who has it if the person just gave it away, and Clear.
#[component]
pub(super) fn ShortcutCell(row: ShortcutRow, drive: EventHandler<ShortcutsInput>) -> Element {
    let shown = if row.shown.is_empty() {
        NO_CHORD.to_owned()
    } else {
        row.shown.clone()
    };
    let action = row.action;
    let clearable = if row.chord.is_some() {
        Availability::Enabled
    } else {
        Availability::Disabled
    };
    rsx! {
        div { class: "shortcut-cell", "data-shortcut": "{row.name}",
            Label { text: shown, role: LabelRole::Primary }
            if let Some(holder) = row.clash.clone() {
                Label { text: format!("Used by {holder}"), role: LabelRole::Tertiary, style: LabelStyle::Footnote }
            }
            Button {
                common: Common {
                    aria_label: Some(format!("Clear {}", row.name)),
                    ..Common::default()
                },
                label: "Clear",
                role: ButtonRole::Normal,
                size: ControlSize::Small,
                availability: clearable,
                onclick: move |_| drive.call(ShortcutsInput::Clear(action)),
            }
        }
    }
}
