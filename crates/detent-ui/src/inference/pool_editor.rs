//! Automatic's list: every model the slot's picker offers, each turned in or out. A model the
//! list holds and the module no longer offers is listed apart, and can be taken out.

use super::note::NoteLine;
use crate::key_row::row_id;
use detent_model::{AutoPool, PickerInput, PoolEntry, PoolMembership, PoolStanding};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::FieldRow;
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::KeyPath;

/// What the list is called.
const TITLE: &str = "Choose from";

/// Said under a model the list holds and the module no longer offers.
const GONE: &str = "No longer available. Turn off to remove.";

/// The list, or nothing when the module offers no list for the slot.
#[component]
pub(super) fn PoolEditor(
    pool: AutoPool,
    availability: Availability,
    onask: EventHandler<PickerInput>,
) -> Element {
    let Some(path) = pool.path.clone() else {
        return rsx! {};
    };
    rsx! {
        div { class: "picker-auto", "data-pool": "editor",
            Label { text: TITLE, role: LabelRole::Secondary }
        }
        for group in pool.groups() {
            if let Some(name) = group.group.clone() {
                div { class: "picker-auto",
                    Label { text: name, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                }
            }
            for entry in group.entries {
                PoolRow { key: "{entry.word}", path: path.clone(), entry, availability, onask }
            }
        }
        if let Some(text) = pool.note.clone() {
            NoteLine { text, kind: "pool" }
        }
    }
}

/// One model: its name and a switch that puts it in the list or takes it out.
#[component]
fn PoolRow(
    path: KeyPath,
    entry: PoolEntry,
    availability: Availability,
    onask: EventHandler<PickerInput>,
) -> Element {
    let on = match entry.membership {
        PoolMembership::In => Check::On,
        PoolMembership::Out => Check::Off,
    };
    let turnable = if entry.can_toggle() {
        availability
    } else {
        Availability::Disabled
    };
    let help = match &entry.standing {
        PoolStanding::Gone => Some(GONE.to_owned()),
        PoolStanding::Offered(gate) => gate.note().map(str::to_owned),
    };
    let word = entry.word.clone();
    rsx! {
        FieldRow {
            label: TextLine::from(entry.label.clone()),
            help: help.map(TextLine::from),
            availability: turnable,
            common: Common { id: Some(row_id("pool", &entry.word)), ..Common::default() },
            Toggle {
                label: entry.label.clone(),
                value: on,
                availability: turnable,
                onchange: move |_| onask.call(PickerInput::PoolToggled { path: path.clone(), word: word.clone() }),
            }
        }
    }
}
