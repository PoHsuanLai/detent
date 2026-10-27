//! The assistant's row of the picker: one pop-up button, the same one the Intelligence page
//! draws for the text slot's balanced tier.

use crate::engine_install::EngineSheetView;
use crate::picker::{TierRow, TurnOnAlert, Unavailable};
use detent_model::{Clearing, ModelRow, PickerInput, PickerState, assistant_of};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;

/// The row, or why it is not there. `state` is the picker's.
#[component]
pub(crate) fn AssistantPicker(
    state: PickerState,
    clearing: Clearing,
    onask: EventHandler<PickerInput>,
) -> Element {
    let ready = match &state {
        PickerState::Ready(ready) => ready,
        PickerState::Unavailable(absence) => {
            return rsx! { Unavailable { absence: *absence } };
        }
        PickerState::Looking => return rsx! {},
    };
    let Some(row) = assistant_of(&state) else {
        return rsx! { Missing {} };
    };
    let row = ModelRow {
        label: crate::copy::common::MODEL.to_owned(),
        help: String::new(),
        ..row.clone()
    };
    rsx! {
        div { class: "picker", "data-picker": "ready",
            FieldGroup {
                TierRow { row, applying: ready.applying().cloned(), adding: ready.adding(), clearing, onask }
            }
            EngineSheetView {}
            TurnOnAlert { ready: ready.clone(), onask }
        }
    }
}

/// The picker is there but lists no model for the assistant.
#[component]
fn Missing() -> Element {
    rsx! {
        div { class: "picker", "data-picker": "missing",
            FieldGroup {
                FieldRow {
                    label: TextLine::from(crate::copy::common::MODEL),
                    help: Some(TextLine::from(crate::copy::assistant_model::THE_MODEL_SERVICE_OFFERS_NO_MODEL)),
                    availability: Availability::Disabled,
                    Label { text: "Unavailable", role: LabelRole::Secondary }
                }
            }
        }
    }
}
