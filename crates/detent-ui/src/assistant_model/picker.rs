//! The assistant's row of the picker: one pop-up button, the same one the Intelligence page
//! draws for the text slot's balanced tier.

use crate::engine_install::EngineSheetView;
use crate::inference::{TierRow, TurnOnAlert, Unavailable};
use detent_model::{Clearing, ModelRow, PickerInput, PickerState, assistant_of};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;

/// What the row is called here; the picker's own name for it is the tier.
const ROW_LABEL: &str = "Model";

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
            return rsx! { Unavailable { absence: absence.clone() } };
        }
        PickerState::Probing => return rsx! {},
    };
    let Some(row) = assistant_of(&state) else {
        return rsx! { Missing {} };
    };
    let row = ModelRow {
        label: ROW_LABEL.to_owned(),
        help: String::new(),
        ..row.clone()
    };
    rsx! {
        div { class: "picker", "data-picker": "ready",
            FieldGroup {
                TierRow { row, applying: ready.applying.clone(), adding: ready.adding(), clearing, onask }
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
                    label: TextLine::from(ROW_LABEL),
                    help: Some(TextLine::from("The model service offers no model for the assistant to use.")),
                    availability: Availability::Disabled,
                    Label { text: "Unavailable", role: LabelRole::Secondary }
                }
            }
        }
    }
}
