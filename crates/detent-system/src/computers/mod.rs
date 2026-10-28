//! The computers over inferd, through porter's client: the places, the Tailscale candidates, the
//! guests and the changes. The engines signal (a computer answering or not) comes from its proxy.

mod bus;
#[cfg(feature = "testing")]
mod fake;

pub use bus::{OnBus, relay};
#[cfg(feature = "testing")]
pub use fake::FakeComputers;

use crate::ticks::Ticks;
use detent_model::{
    Absence, ComputersFound, ComputersReason, ServerDraft, ServerFault, ServerModel, Work,
};
use porter_client::{ComputerName, GuestAnswer, NodeId};
use porter_core::lending::ComputerCandidate;
use std::fmt::Debug;

/// One change to the computers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Add the Tailscale computer.
    Add(NodeId),
    /// Remove the computer.
    Remove(ComputerName),
    /// Answer a guest.
    Answer(NodeId, GuestAnswer),
    /// Forget a guest.
    Forget(NodeId),
    /// Add a model server the person already runs.
    Server(ServerDraft),
}

/// Why a change was not made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeFailure {
    /// The server's form was wrong before anything was sent.
    Fault(ServerFault),
    /// The service refused or could not be reached, for this reason.
    Refused(ComputersReason),
}

/// What the Your computers group asks of inferd.
pub trait ComputersLink: Debug + Send + Sync {
    /// The person's computers and the guests, as inferd lists them now.
    fn read(&self) -> Work<'_, Result<ComputersFound, Absence>>;
    /// The Tailscale computers that can be added, or why not.
    fn candidates(&self) -> Work<'_, Result<Vec<ComputerCandidate>, ComputersReason>>;
    /// The models a server may run, as inferd's picker lists them; none when it cannot be read.
    fn models(&self) -> Work<'_, Vec<ServerModel>>;
    /// Makes one change.
    fn change(&self, change: Change) -> Work<'_, Result<(), ChangeFailure>>;
    /// A tick each time the lists changed, only when inferd is on the bus.
    fn changes(&self) -> Work<'_, Result<Ticks, Absence>>;
}
