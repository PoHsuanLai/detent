//! inferd's model picker, the live module `org.quire.SettingsModule1` at
//! `/org/quire/Inference1/settings` (design/22 section 9.4), as pure state.

mod absence;
mod choice;
mod local;
mod menu;
mod model;
mod pool;
mod step;

pub use absence::Absence;
pub use choice::{Choice, ChoiceGap, ChoiceReadiness, Gate};
pub use local::{LocalAsk, LocalEngine, LocalModel, LocalView};
pub use menu::{Clearing, MenuLine};
pub use model::{
    Applying, AwaitingAccount, ChoiceSection, Described, KindRows, ModelRow, PickerState, Ready,
    TurnOn, loading_text,
};
pub use pool::{AutoPool, PoolEntry, PoolGroup, PoolMembership, PoolStanding, PoolWrite};
pub use step::{PickerEffect, PickerInput, TurnOnAnswer, step};

#[cfg(test)]
mod modes_tests;
#[cfg(test)]
mod place_tests;
#[cfg(test)]
mod tests;
