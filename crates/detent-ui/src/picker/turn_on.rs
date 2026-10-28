//! The question asked when a coding agent is picked while coding agents are switched off.

use detent_model::{PickerInput, Ready};
use dioxus::prelude::*;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;

/// The question while one stands: nothing when no coding agent waits on it.
#[component]
pub(crate) fn TurnOnAlert(ready: Ready, onask: EventHandler<PickerInput>) -> Element {
    let Some(asked) = ready.turning_on() else {
        return rsx! {};
    };
    let name = ready
        .row(&asked.path)
        .map_or_else(|| asked.word.clone(), |row| row.label_of(&asked.word));
    let message = crate::copy::picker::coding_agents_off(&name);
    rsx! {
        Alert {
            title: crate::copy::picker::TURN_ON_CODING_AGENTS,
            message: Some(TextLine::from(message)),
            style: AlertStyle::Informational,
            buttons: vec![
                AlertButton::new(
                    crate::copy::common::TURN_ON,
                    AlertRole::Normal,
                    EventHandler::new(move |()| onask.call(PickerInput::ConfirmTurnOn)),
                ),
                AlertButton::new(
                    crate::copy::common::CANCEL,
                    AlertRole::Cancel,
                    EventHandler::new(move |()| onask.call(PickerInput::CancelTurnOn)),
                ),
            ],
        }
    }
}
