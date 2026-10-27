//! The first-run step's two actions: Skip always, Continue once a model is chosen.

use super::row::assistant_of;
use crate::picker::{ChoiceReadiness, PickerState};

/// What the step can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupAction {
    /// Leave without choosing; the model stays as it is, unset when none was chosen.
    Skip,
    /// Leave with the model that was chosen.
    Continue,
}

/// Whether an action can be taken now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupAvailability {
    /// It can.
    Now,
    /// Not yet.
    NotYet,
}

/// Whether `action` can be taken. Continue waits until the assistant's row can be used (a
/// model, Local with its model, Automatic with a model to choose from), and while a pick is being
/// written or waits on an account; Skip never waits.
pub fn setup_availability(state: &PickerState, action: SetupAction) -> SetupAvailability {
    match action {
        SetupAction::Skip => SetupAvailability::Now,
        SetupAction::Continue => continue_now(state),
    }
}

fn continue_now(state: &PickerState) -> SetupAvailability {
    let PickerState::Ready(ready) = state else {
        return SetupAvailability::NotYet;
    };
    let usable = assistant_of(state).is_some_and(|row| row.readiness() == ChoiceReadiness::Ready);
    if usable && ready.is_idle() {
        SetupAvailability::Now
    } else {
        SetupAvailability::NotYet
    }
}
