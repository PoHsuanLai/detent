//! The person's other computers, as the Intelligence page meets them: the ones the assistant may
//! borrow models from, the Tailscale computers that can be added, and the computers asking to
//! borrow this computer's models.

mod step;
mod words;

pub use step::{
    AddList, ComputerRow, ComputersEffect, ComputersFound, ComputersInput, ComputersReady,
    ComputersState, Standing, computers_step,
};
pub use words::guest_line;

#[cfg(test)]
mod tests;
