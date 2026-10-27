//! One shortcut: its name, who set it, its chord and, for one the person changed, Reset.

use super::field::ShortcutCell;
use crate::copy::shortcuts::{shortcut_note, standard_action_name};
use crate::key_row::row_id;
use detent_model::{Origin, ShortcutRow, ShortcutsInput};
use dioxus::prelude::*;
use ds::components::controls::button_model::Bezel;
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::FieldRow;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// A row of the list.
#[component]
pub(super) fn ShortcutLine(row: ShortcutRow, drive: EventHandler<ShortcutsInput>) -> Element {
    let action = row.action;
    let name = standard_action_name(action);
    rsx! {
        FieldRow {
            label: TextLine::from(name.clone()),
            help: Some(TextLine::from(shortcut_note(&row))),
            help_shown: HelpShown::OnRequest,
            common: Common { id: Some(row_id("shortcut", &name)), ..Common::default() },
            ShortcutCell { row: row.clone(), drive }
            if row.origin == Origin::You {
                Button {
                    common: Common {
                        aria_label: Some(format!("Reset {name}")),
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
