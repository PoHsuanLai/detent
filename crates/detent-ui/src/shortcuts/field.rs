//! How a chord is shown and changed in a row: quire's recorder. The one file that names it.

use detent_model::{ShortcutRow, ShortcutsInput};
use dioxus::prelude::*;
use ds::prelude::*;

/// The recorder of `row`: a chord recorded is set, Delete clears, Escape changes nothing. While
/// the person is asked whether to take a chord over, the recorder says who has it.
#[component]
pub(super) fn ShortcutCell(row: ShortcutRow, drive: EventHandler<ShortcutsInput>) -> Element {
    let action = row.action;
    let clash = row
        .clash
        .clone()
        .map_or(ShortcutClash::None, ShortcutClash::With);
    rsx! {
        ShortcutField {
            label: row.name.clone(),
            value: row.chord,
            clash,
            onrecord: move |recorded: ShortcutRecorded| match recorded {
                ShortcutRecorded::Chord(chord) => drive.call(ShortcutsInput::Set(action, chord)),
                ShortcutRecorded::Cleared => drive.call(ShortcutsInput::Clear(action)),
                _ => {}
            },
        }
    }
}
