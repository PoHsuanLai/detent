//! The question asked when a coding agent is picked while coding agents are switched off.

use detent_model::{PickerInput, Ready, TurnOnAnswer};
use dioxus::prelude::*;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;

/// The button that says yes.
const TURN_ON: &str = "Turn On";

/// The button that says no.
const CANCEL: &str = "Cancel";

/// The question while one stands: nothing when no coding agent waits on it.
#[component]
pub(crate) fn TurnOnAlert(ready: Ready, onask: EventHandler<PickerInput>) -> Element {
    let Some(asked) = ready.turning_on.as_ref() else {
        return rsx! {};
    };
    let name = ready
        .row(&asked.path)
        .map_or_else(|| asked.word.clone(), |row| row.label_of(&asked.word));
    let message = format!(
        "Coding agents are turned off. Turn them on to let the assistant hand your requests to \
         {name}. The desktop still checks what {name} does and asks you when it matters."
    );
    rsx! {
        Alert {
            title: "Turn on coding agents?",
            message: Some(TextLine::from(message)),
            style: AlertStyle::Informational,
            buttons: vec![
                AlertButton::new(
                    TURN_ON,
                    AlertRole::Normal,
                    EventHandler::new(move |()| onask.call(PickerInput::TurnOn(TurnOnAnswer::Yes))),
                ),
                AlertButton::new(
                    CANCEL,
                    AlertRole::Cancel,
                    EventHandler::new(move |()| onask.call(PickerInput::TurnOn(TurnOnAnswer::No))),
                ),
            ],
        }
    }
}
