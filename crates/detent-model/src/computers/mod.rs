//! The person's other computers, as the Intelligence page meets them: the ones the assistant may
//! borrow models from, the Tailscale computers that can be added, and the computers asking to
//! borrow this computer's models.

mod server;
mod step;

pub use server::{
    ComputersReason, KeyField, NAME_MOST, NewServer, ServerDraft, ServerEdit, ServerFault,
    ServerForm, ServerModel, ServerModels, ServerPhase, ServerReach, ServerRefusal, server_models,
};
pub use step::{
    AddList, ComputerRow, ComputersEffect, ComputersFound, ComputersInput, ComputersReady,
    ComputersState, computers_step,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod server_tests;
