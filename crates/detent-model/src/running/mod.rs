//! The models this computer runs itself, as the Intelligence page meets them: what each one is
//! doing in plain words, the memory it takes, and the Start or Stop that changes it. Pure; the
//! calls are `detent-ui`'s.

mod board;
mod words;

pub use board::{RunBoard, RunEffect, RunInput, RunPane, running_step};
pub use words::{InFlight, ModelRun, RunButton, RunRowView, RunStanding, memory_words, model_name};

#[cfg(test)]
mod tests;
