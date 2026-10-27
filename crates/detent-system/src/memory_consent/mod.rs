//! The memory service's controls, as Settings uses them: which looser values wait for the person's
//! word and confirming them, and whether memory keeps to its own folders. None starts the service:
//! with it not running nothing waits on anyone, and opening Settings must not start it.

mod bus;
mod proxy;

pub use bus::OnBus;
pub use proxy::control;

use almanac_core::WaitingSetting;
use detent_model::{Confinement, SettingPair, Trouble, Work};
use std::fmt::Debug;

/// Why a call did not return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryFailure {
    /// There is no bus, or memory is not running: there is nothing to say to the person.
    Absent,
    /// Memory (or the bus) refused or failed, in a way the person is told.
    Trouble(Trouble),
}

/// What the pages that hold memory settings ask of memory.
pub trait MemoryLink: Debug + Send + Sync {
    /// The looser values the file holds that wait for the person, in key order, read fresh.
    fn waiting(&self) -> Work<'_, Result<Vec<WaitingSetting>, MemoryFailure>>;
    /// Tells memory the person agreed to `pairs`; memory takes all of them or none.
    fn confirm(&self, pairs: Vec<SettingPair>) -> Work<'_, Result<(), MemoryFailure>>;
    /// How far memory confines itself, read fresh, only when memory is already running.
    fn confinement(&self) -> Work<'_, Result<Confinement, MemoryFailure>>;
}
