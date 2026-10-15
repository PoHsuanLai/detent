//! The Assistant Model page: what the model is for, and the one row that chooses it.

use super::picker::AssistantPicker;
use crate::inference::{Picker, use_picker};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::prelude::*;

/// What the model is for.
const PURPOSE: &str = "Your assistant uses this model to understand what you ask and to answer. \
    Choose one that runs on this computer, or one from an online company.";

/// The page: the purpose, then the picker's row.
#[component]
pub(crate) fn AssistantModelPane() -> Element {
    let Picker { state, ask } = use_picker();
    rsx! {
        div { class: "assistant-model", "data-assistant": "page",
            div { class: "picker-auto",
                Label { text: PURPOSE, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
            AssistantPicker { state: state(), onask: ask }
        }
    }
}
