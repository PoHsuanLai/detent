//! The first-run step's two buttons: Skip always, Continue once a model is chosen.

use super::row::assistant_of;
use crate::picker::{Choice, PickerState};

/// A button of the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupButton {
    /// Leave without choosing; the model stays as it is, unset when none was chosen.
    Skip,
    /// Leave with the model that was chosen.
    Continue,
}

/// Whether a button can be pressed now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressable {
    /// It can.
    Now,
    /// Not yet.
    NotYet,
}

/// Whether `button` can be pressed. Continue waits until the assistant's row holds a model or
/// Automatic, and while a pick is being written or waits on an account; Skip never waits.
pub fn setup_button(state: &PickerState, button: SetupButton) -> Pressable {
    match button {
        SetupButton::Skip => Pressable::Now,
        SetupButton::Continue => continue_now(state),
    }
}

fn continue_now(state: &PickerState) -> Pressable {
    let PickerState::Ready(ready) = state else {
        return Pressable::NotYet;
    };
    let chosen = assistant_of(state).is_some_and(|row| row.choice() != Choice::Nothing);
    if chosen && ready.is_idle() {
        Pressable::Now
    } else {
        Pressable::NotYet
    }
}
