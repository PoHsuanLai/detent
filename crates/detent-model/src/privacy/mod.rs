//! The Privacy & Security page: which program may use which power, as the compositor lists it.
//! The compositor owns the answers and publishes them as a live settings module; this reads the
//! rows into programs and powers, and turns a choice into the one change the module takes.

mod pane;
mod power;
mod programs;
mod step;

pub use pane::{
    CellKind, PowerCell, PowerStanding, PrivacyPane, PrivacyProgram, ProgramSubject, SubjectKind,
};
pub use power::PrivacyPower;
pub use programs::{DesktopApp, ProgramBook, ProgramLook};
pub use step::{
    PrivacyEffect, PrivacyInput, PrivacyReady, PrivacyState, PrivacySwitch, privacy_step,
};

#[cfg(test)]
mod tests;
