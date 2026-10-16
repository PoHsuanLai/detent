//! How the Software Update page reaches the update service: one handle the environment carries,
//! so a test hands in a service that only answers from memory.

use crate::ticks::Ticks;
use detent_model::{Absence, UnixSeconds, UpdateStatus};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// UpdateWork in flight; the seam's methods are async, and a trait object cannot be.
pub type UpdateWork<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A tick for each time the service's standing may have changed: read it again. Ticks that come
/// while one waits are one tick, and dropping this ends whatever produces them.
pub type UpdateTicks = Ticks;

/// What the page needs of the update service and of the clock.
pub trait UpdatesLink: Send + Sync {
    /// The service's standing now, or why it cannot be read.
    fn read(&self) -> UpdateWork<'_, Result<UpdateStatus, Absence>>;
    /// The ticks that say the standing may have changed, also when the service appears or goes;
    /// only the bus being unreachable is an error.
    fn changes(&self) -> UpdateWork<'_, Result<UpdateTicks, Absence>>;
    /// Asks the service to look for a newer version; the service's words when it refuses.
    fn check(&self) -> UpdateWork<'_, Result<(), String>>;
    /// Asks the service to install the newer version; the service's words when it refuses.
    fn install(&self) -> UpdateWork<'_, Result<(), String>>;
    /// The time "last checked" counts from.
    fn now(&self) -> UnixSeconds;
}

/// The update service, as the environment carries it.
#[derive(Clone)]
pub struct Updater(Arc<dyn UpdatesLink>);

impl fmt::Debug for Updater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Updater").finish_non_exhaustive()
    }
}

impl Updater {
    /// The service over `link`: what a test hands in.
    pub fn over(link: impl UpdatesLink + 'static) -> Updater {
        Updater(Arc::new(link))
    }

    pub(crate) fn read(&self) -> UpdateWork<'_, Result<UpdateStatus, Absence>> {
        self.0.read()
    }

    pub(crate) fn changes(&self) -> UpdateWork<'_, Result<UpdateTicks, Absence>> {
        self.0.changes()
    }

    pub(crate) fn check(&self) -> UpdateWork<'_, Result<(), String>> {
        self.0.check()
    }

    pub(crate) fn install(&self) -> UpdateWork<'_, Result<(), String>> {
        self.0.install()
    }

    pub(crate) fn now(&self) -> UnixSeconds {
        self.0.now()
    }
}
