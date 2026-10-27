//! The models this computer runs itself, as the Intelligence page meets them: what each one is
//! doing, the memory it takes, and the start or stop that changes it. Pure; the calls and the
//! words are `detent-ui`'s.

mod board;
mod model;

pub use board::{RunBoard, RunEffect, RunInput, RunPane, running_step};
pub use model::{InFlight, ModelRun, RunAction, RunRowView, RunStanding};

#[cfg(test)]
mod tests;
