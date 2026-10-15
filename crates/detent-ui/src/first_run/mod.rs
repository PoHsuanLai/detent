//! `detent --first-run`: the short window that asks whether AI stays on this computer and shows
//! the assistant's model row, once. It writes `ai.local_only` through the settings path the
//! Intelligence page writes through, the model through the picker as it is chosen, and records
//! that it ran under the state home.

mod io;
mod view;

pub use view::FirstRunApp;
