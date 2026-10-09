//! The desktop-wide Spaces, as the Spaces page meets them: the list, and the sheet that asks what
//! becomes of a Space's memories and history before it is removed.

mod removal;
mod step;

pub use removal::{HISTORY_CHOICES, HISTORY_NOTE, RemovalSheet, SheetInput};
pub use step::{
    SpaceKey, SpaceRow, SpacesEffect, SpacesInput, SpacesReady, SpacesState, spaces_step,
};

#[cfg(test)]
mod tests;
