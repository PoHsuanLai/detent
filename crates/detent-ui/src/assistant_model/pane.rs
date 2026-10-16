//! The Assistant Model page: the one row that chooses the model.

use super::picker::AssistantPicker;
use crate::inference::{Picker, use_picker};
use detent_model::Clearing;
use dioxus::prelude::*;

/// The page: the picker's row.
#[component]
pub(crate) fn AssistantModelPane() -> Element {
    let Picker { state, ask } = use_picker();
    rsx! {
        div { class: "assistant-model", "data-assistant": "page",
            AssistantPicker { state: state(), clearing: Clearing::Offered, onask: ask }
        }
    }
}
