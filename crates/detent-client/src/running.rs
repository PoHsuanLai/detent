//! The Local models list: the models inferd runs itself, read again each time one changes state
//! or inferd comes back. Looking never starts the assistant. The steps are `detent-model`'s
//! `running_step`.

use crate::live::Live;
use crate::pane::{Machine, Pending, pane};
use detent_model::{RunAction, RunEffect, RunInput, RunPane, running_step};
use detent_system::bus::{BusSource, Daemon, connect, next_owner, owner_changes};
use detent_system::running::RunningLink;
use std::sync::Arc;

/// The running models' handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Running {
    bus: BusSource,
    link: Arc<dyn RunningLink>,
    live: Live<RunPane>,
}

pane!(Running, RunPane, follow);

impl Running {
    pub(crate) fn over(bus: BusSource, link: Arc<dyn RunningLink>) -> Running {
        Running {
            bus,
            link,
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: RunInput) -> Pending {
        self.dispatch(input)
    }

    /// Reads the list, then reads it again each time a model changes state or inferd comes back.
    async fn follow(&self) {
        let connection = match connect(&self.bus).await {
            Ok(connection) => connection,
            Err(absence) => return self.dispatch(RunInput::Found(Err(absence))).await,
        };
        let Ok(mut owners) = owner_changes(&connection, &Daemon::inferd()).await else {
            return;
        };
        loop {
            // Subscribed before the first read, so a change in between is not lost.
            let watch = self.link.changes().await;
            self.reread().await;
            if let Ok(mut watch) = watch {
                loop {
                    tokio::select! {
                        changed = watch.recv() => match changed {
                            Some(()) => self.reread().await,
                            None => break,
                        },
                        owner = next_owner(&mut owners) => match owner {
                            Some(()) => break,
                            None => return,
                        },
                    }
                }
                continue;
            }
            if next_owner(&mut owners).await.is_none() {
                return;
            }
        }
    }

    async fn reread(&self) {
        self.dispatch(RunInput::Found(self.link.read().await)).await;
    }
}

impl Machine for Running {
    type Input = RunInput;
    type Effect = RunEffect;

    fn step(state: RunPane, input: RunInput) -> (RunPane, Vec<RunEffect>) {
        running_step(state, input)
    }

    async fn run(&self, effect: RunEffect) -> Vec<RunInput> {
        match effect {
            RunEffect::Look => vec![RunInput::Found(self.link.read().await)],
            RunEffect::Start(model) => {
                let ended = self.link.act(model.clone(), RunAction::Start).await;
                vec![RunInput::Answered(model, ended)]
            }
            RunEffect::Stop(model) => {
                let ended = self.link.act(model.clone(), RunAction::Stop).await;
                vec![RunInput::Answered(model, ended)]
            }
        }
    }
}
