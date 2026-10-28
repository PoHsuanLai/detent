//! Lending this computer's models to the person's other computers, as the Settings switch meets
//! it: turning it on asks first, then the sharing is set up, and only when that worked is the
//! setting written. Pure; the file and the service are the window's (`detent-ui`).

mod step;

pub use step::{
    Existing, LendFailure, SHARING_KEY, Sharing, SharingEffect, SharingInput, Way, sharing_step,
};

#[cfg(test)]
mod tests;
