//! `detent --first-run`: the short window that asks whether AI stays on this computer and which
//! local runtime to use, once. It writes through the settings path the Intelligence page writes
//! through and records that it ran under the state home.

mod io;
mod view;

pub use view::FirstRunApp;
