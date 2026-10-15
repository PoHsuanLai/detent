//! inferd's model picker, the live module `org.quire.SettingsModule1` at
//! `/org/quire/Inference1/settings` (design/22 section 9.4), as pure state.

mod choice;
mod model;
mod step;

pub use choice::{Choice, Gate};
pub use model::{
    Absence, Applying, AwaitingAccount, ChoiceSection, Described, KindRows, ModelRow, PickerState,
    Ready, loading_text,
};
pub use step::{PickerEffect, PickerInput, step};

#[cfg(test)]
mod tests;
