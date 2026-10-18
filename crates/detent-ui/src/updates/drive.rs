//! The Software Update page's state and what it needs to act: following the service, asking it,
//! and starting the log-out. The steps are `detent-model`'s `updates_step`.

use super::leave::Departure;
use super::link::{InstallRefusal, UpdatesLink};
use crate::env::Env;
use detent_model::{UpdatesEffect, UpdatesInput, UpdatesState, updates_step};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::sync::Arc;

/// The page's state and its links.
#[derive(Clone)]
pub(super) struct Driver {
    updater: Arc<dyn UpdatesLink>,
    log_out: Arc<dyn Departure>,
    toasts: ToastHub,
    pub(super) state: Signal<UpdatesState>,
}

/// The page's state, following the service for as long as the page is shown.
pub(super) fn use_updates() -> Driver {
    let env = use_context::<Env>();
    let standing = Driver {
        updater: env.updates.clone(),
        log_out: env.log_out.clone(),
        toasts: use_toasts(),
        state: use_signal(UpdatesState::default),
    };
    use_future({
        let standing = standing.clone();
        move || {
            let standing = standing.clone();
            async move { standing.follow().await }
        }
    });
    standing
}

impl Driver {
    /// Reads the service now and each time it may have changed; the service going away or coming
    /// back is a change too.
    async fn follow(&self) {
        match self.updater.changes().await {
            Err(absence) => self.feed(UpdatesInput::Heard(Err(absence))),
            Ok(mut ticks) => {
                self.reread().await;
                while ticks.recv().await.is_some() {
                    self.reread().await;
                }
            }
        }
    }

    async fn reread(&self) {
        self.feed(UpdatesInput::Heard(self.updater.read().await));
    }

    /// Gives the page `input` and carries out what it asks for.
    pub(super) fn feed(&self, input: UpdatesInput) {
        let mut state = self.state;
        let (next, effects) = updates_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            self.run(effect);
        }
    }

    fn run(&self, effect: UpdatesEffect) {
        let me = self.clone();
        match effect {
            UpdatesEffect::Check => {
                spawn(async move {
                    let done = me.updater.check().await;
                    me.answered(done).await;
                });
            }
            UpdatesEffect::Install(version) => {
                spawn(async move {
                    let done = me.updater.install(&version).await;
                    me.installed(done).await;
                });
            }
            UpdatesEffect::LogOut => {
                spawn(async move {
                    if let Err(words) = me.log_out.log_out().await {
                        me.toasts.push(words, None);
                    }
                });
            }
            UpdatesEffect::Tell(words) => self.toasts.push(words, None),
        }
    }

    /// The service took a request (read what it says now) or refused it (say why).
    async fn answered(&self, done: Result<(), String>) {
        match done {
            Ok(()) => self.reread().await,
            Err(words) => self.feed(UpdatesInput::Refused(words)),
        }
    }

    /// The service took the install (read what it says now), or would not: it offers another
    /// version now (read it, then say so beside it), or it refused (say why).
    async fn installed(&self, done: Result<(), InstallRefusal>) {
        match done {
            Ok(()) => self.reread().await,
            Err(InstallRefusal::VersionChanged) => {
                self.reread().await;
                self.feed(UpdatesInput::VersionChanged);
            }
            Err(InstallRefusal::Told(words)) => self.feed(UpdatesInput::Refused(words)),
        }
    }
}
