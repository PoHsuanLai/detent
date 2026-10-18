//! How the Software Update page reaches the update service: one handle the environment carries,
//! so a test hands in a service that only answers from memory.

use crate::ticks::Ticks;
use detent_model::{Absence, UnixSeconds, UpdateStatus, UpdateVersion, Work};
use std::fmt::Debug;

/// A tick for each time the service's standing may have changed: read it again. Ticks that come
/// while one waits are one tick, and dropping this ends whatever produces them.
pub type UpdateTicks = Ticks;

/// Why the service did not take an install request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallRefusal {
    /// What it offers is not the version that was named, or it offers nothing now.
    VersionChanged,
    /// It refused or failed, in these words for the person.
    Told(String),
}

/// What the page needs of the update service and of the clock.
pub trait UpdatesLink: Debug + Send + Sync {
    /// The service's standing now, or why it cannot be read.
    fn read(&self) -> Work<'_, Result<UpdateStatus, Absence>>;
    /// The ticks that say the standing may have changed, also when the service appears or goes;
    /// only the bus being unreachable is an error.
    fn changes(&self) -> Work<'_, Result<UpdateTicks, Absence>>;
    /// Asks the service to look for a newer version; the service's words when it refuses.
    fn check(&self) -> Work<'_, Result<(), String>>;
    /// Asks the service to install `version`, the one the page shows; it refuses when what it
    /// offers is another.
    fn install(&self, version: &UpdateVersion) -> Work<'_, Result<(), InstallRefusal>>;
    /// The time "last checked" counts from.
    fn now(&self) -> UnixSeconds;
}
