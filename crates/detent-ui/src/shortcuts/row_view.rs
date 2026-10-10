//! One shortcut: its name, who set it, its chord and, for one the person changed, Reset.

use super::field::ShortcutCell;
use detent_model::{Origin, ShortcutRow, ShortcutsInput};
use dioxus::prelude::*;
use ds::components::controls::button_model::Bezel;
use ds::components::fields::field_row::FieldRow;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// A row of the list.
#[component]
pub(super) fn ShortcutLine(row: ShortcutRow, drive: EventHandler<ShortcutsInput>) -> Element {
    let action = row.action;
    rsx! {
        FieldRow {
            label: TextLine::from(row.name.clone()),
            help: Some(TextLine::from(row.note())),
            ShortcutCell { row: row.clone(), drive }
            if row.origin == Origin::You {
                Button {
                    common: Common {
                        aria_label: Some(format!("Reset {}", row.name)),
                        ..Common::default()
                    },
                    label: "Reset",
                    bezel: Bezel::Inline,
                    size: ControlSize::Small,
                    onclick: move |_| drive.call(ShortcutsInput::Reset(action)),
                }
            }
        }
    }
}
