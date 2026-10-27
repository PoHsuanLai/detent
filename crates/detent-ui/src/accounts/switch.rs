//! A switch's state as the toggle draws it and reports it.

use detent_model::SwitchState;
use ds::prelude::*;

/// How the toggle draws `state`.
pub(super) fn check_of(state: SwitchState) -> Check {
    match state {
        SwitchState::On => Check::On,
        SwitchState::Off => Check::Off,
    }
}

/// The state the toggle's new value asks for.
pub(super) fn switch_of(check: Check) -> SwitchState {
    if check == Check::On {
        SwitchState::On
    } else {
        SwitchState::Off
    }
}
