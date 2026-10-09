//! The person's other computers, as the Intelligence page meets them: the ones the assistant may
//! borrow models from, the Tailscale computers that can be added, and the computers asking to
//! borrow this computer's models.

mod step;
mod words;

pub use step::{
    AddList, Approval, Candidate, ComputerRow, ComputersEffect, ComputersFound, ComputersInput,
    ComputersReady, ComputersState, Guest, GuestAnswer, GuestState, NodeKey, Standing,
    computers_step,
};
pub use words::{guest_line, refusal_sentence};

#[cfg(test)]
mod tests;
