//! The person's other computers, as the Intelligence page meets them: the ones the assistant may
//! borrow models from, the Tailscale computers that can be added, and the computers asking to
//! borrow this computer's models.

mod server;
mod step;
mod words;

pub use server::{
    KeyField, NAME_MOST, ServerDraft, ServerEdit, ServerForm, ServerModel, ServerModels,
    ServerPhase, ServerReach, ServerSheet, server_models,
};
pub use step::{
    AddList, ComputerRow, ComputersEffect, ComputersFound, ComputersInput, ComputersReady,
    ComputersState, Standing, computers_step,
};
pub use words::{ServerFault, guest_line};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod server_tests;
