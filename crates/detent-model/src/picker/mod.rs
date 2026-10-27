//! inferd's model picker, the live module `org.quire.SettingsModule1` at
//! `/org/quire/Inference1/settings` (design/22 section 9.4), as pure state.

mod choice;
mod local;
mod menu;
mod model;
mod pool;
mod step;
mod valued;

pub use choice::{Choice, ChoiceGap, ChoiceReadiness, Gate};
pub use local::{LocalAsk, LocalEngine, LocalModel, LocalView};
pub use menu::{Clearing, MenuLine};
pub use model::{
    Applying, AwaitingAccount, Busy, ChoiceSection, Described, KindRows, Loading, ModelRow,
    PickerState, Ready, TurnOn, loading_of,
};
pub use pool::{
    AutoPool, PoolEntry, PoolGroup, PoolHeading, PoolMembership, PoolStanding, PoolWrite,
};
pub use step::{PickerEffect, PickerInput, TurnOnAnswer, step};
pub use valued::picker_row_is_choice;

#[cfg(test)]
mod gate_tests;
#[cfg(test)]
mod local_tests;
#[cfg(test)]
mod modes_tests;
#[cfg(test)]
mod place_tests;
#[cfg(test)]
mod tests;
