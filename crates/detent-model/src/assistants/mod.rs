//! The built-in assistant's places: where its requests may run, as pure state over docket's
//! typed setting, and the accounts that make a cloud place.

mod known;
mod offers;
mod step;

pub use known::known_of;
pub use offers::{ModelOffer, offers_for};
pub use step::{Confirm, PlaceRef, PlacesEffect, PlacesInput, PlacesState, places_step};

#[cfg(test)]
mod tests;
