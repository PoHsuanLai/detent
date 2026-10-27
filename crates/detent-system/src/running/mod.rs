//! The models inferd runs itself: reading them, starting and stopping one, and the signal that
//! says to read again. Looking never starts the assistant.

mod bus;

pub use bus::OnBus;

use crate::ticks::Ticks;
use detent_model::{Absence, ModelRun, RunAction, Work};
use std::fmt::Debug;

/// What the running-models list asks of the assistant.
pub trait RunningLink: Debug + Send + Sync {
    /// The models inferd runs itself, as it lists them now.
    fn read(&self) -> Work<'_, Result<Vec<ModelRun>, Absence>>;
    /// Does `action` to `model`; the words it was refused with when it was.
    fn act(&self, model: String, action: RunAction) -> Work<'_, Result<(), String>>;
    /// A tick each time a model changed state, only when inferd is on the bus.
    fn changes(&self) -> Work<'_, Result<Ticks, Absence>>;
}
