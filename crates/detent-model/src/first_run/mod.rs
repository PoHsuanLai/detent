//! The first-run window (`detent --first-run`) as pure state: keep AI on this computer, pick a
//! local runtime if one was found, Done.

mod model;
mod step;
mod target;

pub use model::{FirstRun, LocalOnly, Runtime, Stage};
pub use step::{FirstRunEffect, FirstRunInput, first_run_step};
pub use target::runtime_choice;

#[cfg(test)]
mod tests;
