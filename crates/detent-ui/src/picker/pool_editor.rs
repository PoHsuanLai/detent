//! Automatic's list: every model the slot's picker offers, each turned in or out. A model the
//! list holds and the module no longer offers is listed apart, and can be taken out.

use super::note::NoteLine;
use crate::copy::picker as copy;
use crate::key_row::row_id;
use detent_model::{AutoPool, PickerInput, PoolEntry, PoolStanding};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::FieldRow;
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::KeyPath;

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
            Label { text: crate::copy::picker::CHOOSE_FROM, role: LabelRole::Secondary }
        }
        for group in pool.groups() {
            if let Some(name) = copy::pool_heading(&group.heading) {
                div { class: "picker-auto",
                    Label { text: name.to_owned(), role: LabelRole::Tertiary, style: LabelStyle::Footnote }
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
    let on = if entry.in_pool { Check::On } else { Check::Off };
    let turnable = if entry.can_toggle() {
        availability
    } else {
        Availability::Disabled
    };
    let help = match &entry.standing {
        PoolStanding::Gone => Some(crate::copy::picker::GONE_TURN_OFF.to_owned()),
        PoolStanding::Offered(gate) => copy::gate_note(gate),
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
