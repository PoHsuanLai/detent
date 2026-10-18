//! The update service and the log-out, from memory: nothing here reaches a bus, the clock or a
//! process.

use detent_model::{Absence, UnixSeconds, UpdatePhase, UpdateStatus, UpdateVersion, Work};
use detent_ui::{Departure, InstallRefusal, Ticker, UpdateTicks, UpdatesLink};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// What the page asked of the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    Check,
    /// An install, naming this version.
    Install(UpdateVersion),
}

/// The time the page counts "last checked" from.
pub const NOW: UnixSeconds = UnixSeconds(1_000_000);

#[derive(Debug)]
struct Shared {
    standing: Mutex<Result<UpdateStatus, Absence>>,
    asked: Mutex<Vec<Asked>>,
    refusal: Mutex<Option<String>>,
    listeners: Mutex<Vec<Ticker>>,
}

/// A service that answers from what it was given, and writes down what it was asked.
#[derive(Debug, Clone)]
pub struct Service(Arc<Shared>);

impl Service {
    /// A service in this standing.
    pub fn answering(status: UpdateStatus) -> Service {
        Service::standing(Ok(status))
    }

    /// No service on the bus.
    pub fn absent() -> Service {
        Service::standing(Err(Absence::NotRunning))
    }

    fn standing(standing: Result<UpdateStatus, Absence>) -> Service {
        Service(Arc::new(Shared {
            standing: Mutex::new(standing),
            asked: Mutex::default(),
            refusal: Mutex::default(),
            listeners: Mutex::default(),
        }))
    }

    /// The service moves to `status`, and tells whoever follows it.
    pub fn becomes(&self, status: UpdateStatus) {
        *self.0.standing.lock().expect("lock") = Ok(status);
        self.tick();
    }

    /// The service goes off the bus.
    pub fn goes(&self) {
        *self.0.standing.lock().expect("lock") = Err(Absence::NotRunning);
        self.tick();
    }

    /// The service moves to `status` without telling anyone, as when a release lands between the
    /// page reading the service and the person pressing: the page still shows the old standing.
    pub fn changes_unseen(&self, status: UpdateStatus) {
        *self.0.standing.lock().expect("lock") = Ok(status);
    }

    /// The service refuses the next requests, in these words.
    pub fn refuses(&self, words: &str) {
        *self.0.refusal.lock().expect("lock") = Some(words.to_owned());
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<Asked> {
        self.0.asked.lock().expect("lock").clone()
    }

    fn tick(&self) {
        self.0
            .listeners
            .lock()
            .expect("lock")
            .retain(|listener| listener.tick().is_ok());
    }

    /// Takes a request: written down, refused or moved to `working`.
    fn request(&self, asked: Asked, working: UpdatePhase) -> Result<(), String> {
        self.0.asked.lock().expect("lock").push(asked);
        self.admit(working)
    }

    /// Refuses in the words it was given, else moves to `working`.
    fn admit(&self, working: UpdatePhase) -> Result<(), String> {
        if let Some(words) = self.0.refusal.lock().expect("lock").clone() {
            return Err(words);
        }
        let moved = self.0.standing.lock().expect("lock").clone();
        if let Ok(status) = moved {
            self.becomes(UpdateStatus {
                phase: working,
                ..status
            });
        }
        Ok(())
    }

    /// Whether `version` is the one on offer, as the real service checks it.
    fn offers(&self, version: &UpdateVersion) -> bool {
        matches!(
            &*self.0.standing.lock().expect("lock"),
            Ok(UpdateStatus { offer: Some(offer), .. }) if offer.version == *version
        )
    }
}

impl UpdatesLink for Service {
    fn read(&self) -> Work<'_, Result<UpdateStatus, Absence>> {
        let standing = self.0.standing.lock().expect("lock").clone();
        Box::pin(std::future::ready(standing))
    }

    fn changes(&self) -> Work<'_, Result<UpdateTicks, Absence>> {
        let (listener, ticks) = UpdateTicks::channel();
        self.0.listeners.lock().expect("lock").push(listener);
        Box::pin(std::future::ready(Ok(ticks)))
    }

    fn check(&self) -> Work<'_, Result<(), String>> {
        Box::pin(std::future::ready(
            self.request(Asked::Check, UpdatePhase::Checking),
        ))
    }

    fn install(&self, version: &UpdateVersion) -> Work<'_, Result<(), InstallRefusal>> {
        self.0
            .asked
            .lock()
            .expect("lock")
            .push(Asked::Install(version.clone()));
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
pub struct Leaves(Arc<AtomicUsize>);

impl Leaves {
    /// How many times the page started logging out.
    pub fn started(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

impl Departure for Leaves {
    fn log_out(&self) -> Work<'_, Result<(), String>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(std::future::ready(Ok(())))
    }
}
