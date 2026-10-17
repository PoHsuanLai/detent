//! The Privacy & Security page: the programs the compositor lists and the powers each may use, over
//! its live settings module. Settings looks for the compositor and never starts it.

mod book;
mod io;
mod pane;
mod program_view;

pub(crate) use book::use_program_book;
pub(crate) use pane::PrivacyPane;
