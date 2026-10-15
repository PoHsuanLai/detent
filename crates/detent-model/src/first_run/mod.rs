//! The first-run window (`detent --first-run`) as pure state: keep AI on this computer, Done.
//! The assistant's model is the picker's own row, drawn in the window and written as it is chosen.

mod model;
mod step;

pub use model::{FirstRun, LocalOnly, Runtime, Stage};
pub use step::{FirstRunEffect, FirstRunInput, first_run_step};

#[cfg(test)]
mod tests;
