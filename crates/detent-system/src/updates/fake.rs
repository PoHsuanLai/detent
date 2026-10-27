//! The update service and the log-out from memory: nothing here reaches a bus, the clock or a
//! process.

use super::{Departure, InstallRefusal, UpdateTicks, UpdatesLink};
use crate::fake_kit::Kit;
use detent_model::{Absence, UnixSeconds, UpdatePhase, UpdateStatus, UpdateVersion, Work};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// What was asked of the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatesAsked {
    /// A look for a newer version.
    Check,
    /// An install, naming this version.
    Install(UpdateVersion),
}

/// The time "last checked" counts from in the fake.
pub const NOW: UnixSeconds = UnixSeconds(1_000_000);

/// A service that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeUpdates(Arc<Kit<UpdateStatus, UpdatesAsked>>);

impl FakeUpdates {
    /// A service in this standing.
    pub fn answering(status: UpdateStatus) -> FakeUpdates {
        FakeUpdates(Kit::new(Ok(status)))
    }

    /// No service on the bus.
    pub fn absent() -> FakeUpdates {
        FakeUpdates(Kit::new(Err(Absence::NotRunning)))
    }

    /// The service moves to `status` and tells whoever follows it.
    pub fn becomes(&self, status: UpdateStatus) {
        self.0.set(Ok(status));
    }

    /// The service refuses the next requests, in these words.
    pub fn refuses(&self, words: &str) {
        self.0.refuse_with(Some(words.to_owned()));
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<UpdatesAsked> {
        self.0.asked()
    }

    /// Takes a request: refused, or the phase moves to `working`.
    fn admit(&self, working: UpdatePhase) -> Result<(), String> {
        self.0.admit()?;
        self.0.edit(|status| status.phase = working);
        Ok(())
    }

    fn offers(&self, version: &UpdateVersion) -> bool {
        matches!(
            self.0.standing(),
            Ok(UpdateStatus { offer: Some(offer), .. }) if offer.version == *version
        )
    }
}

impl UpdatesLink for FakeUpdates {
    fn read(&self) -> Work<'_, Result<UpdateStatus, Absence>> {
        Box::pin(std::future::ready(self.0.standing()))
    }

    fn changes(&self) -> Work<'_, Result<UpdateTicks, Absence>> {
        Box::pin(std::future::ready(Ok(self.0.listen())))
    }

    fn check(&self) -> Work<'_, Result<(), String>> {
        self.0.record(UpdatesAsked::Check);
        Box::pin(std::future::ready(self.admit(UpdatePhase::Checking)))
    }

    fn install(&self, version: &UpdateVersion) -> Work<'_, Result<(), InstallRefusal>> {
        self.0.record(UpdatesAsked::Install(version.clone()));
        let done = if self.offers(version) {
            self.admit(UpdatePhase::Downloading)
                .map_err(InstallRefusal::Told)
        } else {
            Err(InstallRefusal::VersionChanged)
        };
        Box::pin(std::future::ready(done))
    }

    fn now(&self) -> UnixSeconds {
        NOW
    }
}

/// A log-out that is only counted.
#[derive(Debug, Clone, Default)]
pub struct FakeLeaves(Arc<AtomicUsize>);

impl FakeLeaves {
    /// How many times the log-out was started.
    pub fn started(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

impl Departure for FakeLeaves {
    fn log_out(&self) -> Work<'_, Result<(), String>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(std::future::ready(Ok(())))
    }
}
