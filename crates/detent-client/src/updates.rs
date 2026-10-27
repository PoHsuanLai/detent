//! The Software Update pane: following the service, asking it, and starting the log-out. The steps
//! are `detent-model`'s `updates_step`.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{UnixSeconds, UpdatesEffect, UpdatesInput, UpdatesState, updates_step};
use detent_system::updates::{Departure, InstallRefusal, UpdatesLink};
use std::sync::Arc;

/// The Software Update pane's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Updates {
    link: Arc<dyn UpdatesLink>,
    log_out: Arc<dyn Departure>,
    live: Live<UpdatesState>,
}

pane!(Updates, UpdatesState, follow);

impl Updates {
    pub(crate) fn over(link: Arc<dyn UpdatesLink>, log_out: Arc<dyn Departure>) -> Updates {
        Updates {
            link,
            log_out,
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: UpdatesInput) -> Pending {
        self.dispatch(input)
    }

    /// The time "last checked" counts from.
    pub fn now(&self) -> UnixSeconds {
        self.link.now()
    }

    /// Reads the service now and each time it may have changed; the service going away or coming
    /// back is a change too.
    async fn follow(&self) {
        match self.link.changes().await {
            Err(absence) => self.dispatch(UpdatesInput::Heard(Err(absence))).await,
            Ok(mut ticks) => {
                self.reread().await;
                while ticks.recv().await.is_some() {
                    self.reread().await;
                }
            }
        }
    }

    async fn reread(&self) {
        self.dispatch(UpdatesInput::Heard(self.link.read().await))
            .await;
    }

    /// The service took a request (read what it says now) or refused it (say why).
    async fn answered(&self, done: Result<(), String>) -> Vec<UpdatesInput> {
        match done {
            Ok(()) => vec![UpdatesInput::Heard(self.link.read().await)],
            Err(words) => vec![UpdatesInput::Refused(words)],
        }
    }

    /// The service took the install (read what it says now), or would not: it offers another
    /// version now (read it, then say so beside it), or it refused (say why).
    async fn installed(&self, done: Result<(), InstallRefusal>) -> Vec<UpdatesInput> {
        match done {
            Ok(()) => vec![UpdatesInput::Heard(self.link.read().await)],
            Err(InstallRefusal::VersionChanged) => vec![
                UpdatesInput::Heard(self.link.read().await),
                UpdatesInput::VersionChanged,
            ],
            Err(InstallRefusal::Told(words)) => vec![UpdatesInput::Refused(words)],
        }
    }
}

impl Machine for Updates {
    type Input = UpdatesInput;
    type Effect = UpdatesEffect;

    fn step(state: UpdatesState, input: UpdatesInput) -> (UpdatesState, Vec<UpdatesEffect>) {
        updates_step(state, input)
    }

    async fn run(&self, effect: UpdatesEffect) -> Vec<UpdatesInput> {
        match effect {
            UpdatesEffect::Check => {
                let done = self.link.check().await;
                self.answered(done).await
            }
            UpdatesEffect::Install(version) => {
                let done = self.link.install(&version).await;
                self.installed(done).await
            }
            UpdatesEffect::LogOut => {
                if let Err(words) = self.log_out.log_out().await {
                    self.live.notice(Notice::Words(words));
                }
                Vec::new()
            }
            UpdatesEffect::Tell(words) => {
                self.live.notice(Notice::Words(words));
                Vec::new()
            }
        }
    }
}
