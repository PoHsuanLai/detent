//! The one control that draws a key, and which values it accepts.

mod model;
mod toggle;

pub use model::{Control, FileOnly};
pub use toggle::TogglePair;

#[cfg(test)]
mod tests;
