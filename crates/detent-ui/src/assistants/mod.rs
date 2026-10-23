//! The Intelligence page's assistants: where the built-in assistant may run, and the assistants
//! that sign themselves in.

mod add_flow;
mod add_view;
mod models;
mod outside_view;
mod place_row;
mod places_view;

pub(crate) use place_row::{PlaceEntry, PlaceRow};
pub(crate) use places_view::Assistants;
