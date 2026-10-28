//! The desktop-wide Spaces, as the Spaces page meets them: the list, and the question that asks what
//! becomes of a Space's memories and history before it is removed.

mod removal;
mod step;

pub use removal::{RemovalAsk, RemovalInput, needs_memory};
pub use step::{
    RemovalStage, SpaceKey, SpaceRow, SpacesEffect, SpacesInput, SpacesReady, SpacesState,
    spaces_step,
};

#[cfg(test)]
mod tests;
