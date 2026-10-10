//! The row's chord field over quire's recorder. It takes the place of `field.rs` (same component,
//! same props) once quire has `ShortcutField`, `Recorded` and `Clash`; to switch, replace
//! `field.rs` with this file. Written against the recorder's announced shape:
//! `ShortcutField { value: Option<Chord>, onrecord: EventHandler<Recorded>, clash: Clash }`,
//! `Recorded::{Chord(Chord), Cleared, Cancelled}`, `Clash::{None, With(String)}`. The paths in the
//! `use` lines below are quire's to confirm.

use chordkit::Chord;
use detent_model::{ShortcutRow, ShortcutsInput};
use dioxus::prelude::*;
use ds::components::fields::shortcut_field::{Clash, Recorded, ShortcutField};

/// The recorder of `row`: a chord recorded is set, Delete clears, Escape changes nothing. While
/// the person is asked whether to take a chord over, the recorder says who has it.
#[component]
pub(super) fn ShortcutCell(row: ShortcutRow, drive: EventHandler<ShortcutsInput>) -> Element {
    let action = row.action;
    let clash = match row.clash.clone() {
        Some(holder) => Clash::With(holder),
        None => Clash::None,
    };
    let value: Option<Chord> = row.chord;
    rsx! {
        ShortcutField {
            value,
            clash,
            onrecord: move |recorded| match recorded {
                Recorded::Chord(chord) => drive.call(ShortcutsInput::Set(action, chord)),
                Recorded::Cleared => drive.call(ShortcutsInput::Clear(action)),
                Recorded::Cancelled => {}
            },
        }
    }
}
