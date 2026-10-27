//! Your computers: the person's computers over inferd, the Add list, the guests, and the model
//! servers added by hand. The steps are `detent-model`'s `computers_step`.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{ComputersEffect, ComputersInput, ComputersState, ServerFault, computers_step};
use detent_system::bus::{BusSource, Daemon, connect, next_owner, owner_changes};
use detent_system::computers::{Change, ChangeFailure, ComputersLink};
use std::sync::Arc;

/// The words of a server form's fault, which only the window knows how to say.
pub type FaultWords = fn(ServerFault) -> &'static str;

/// The computers' handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Computers {
    bus: BusSource,
    link: Arc<dyn ComputersLink>,
    words: FaultWords,
    live: Live<ComputersState>,
}

pane!(Computers, ComputersState, follow);

impl Computers {
    pub(crate) fn over(
        bus: BusSource,
        link: Arc<dyn ComputersLink>,
        words: FaultWords,
    ) -> Computers {
        Computers {
            bus,
            link,
            words,
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: ComputersInput) -> Pending {
        self.dispatch(input)
    }

    /// Reads the lists, then reads them again each time inferd says they changed or comes back.
    async fn follow(&self) {
        let connection = match connect(&self.bus).await {
            Ok(connection) => connection,
            Err(absence) => return self.dispatch(ComputersInput::Found(Err(absence))).await,
        };
        let Ok(mut owners) = owner_changes(&connection, &Daemon::inferd()).await else {
            return;
        };
        loop {
            // Subscribed before the first read, so a change in between is not lost.
            let watch = self.link.changes().await;
            self.reread().await;
            if let Ok(mut watch) = watch {
                while watch.recv().await.is_some() {
                    self.reread().await;
                }
            }
            if next_owner(&mut owners).await.is_none() {
                return;
            }
        }
    }

    async fn reread(&self) {
        self.dispatch(ComputersInput::Found(self.link.read().await))
            .await;
    }

    /// The sentence for a change that was not made: the form's own, or the service's.
    fn sentence_of(&self, failure: ChangeFailure) -> String {
        match failure {
            ChangeFailure::Fault(fault) => (self.words)(fault).to_owned(),
            ChangeFailure::Told(words) => words,
        }
    }

    async fn changed(&self, change: Change) -> ComputersInput {
        let ended = self
            .link
            .change(change)
            .await
            .map_err(|failure| self.sentence_of(failure));
        ComputersInput::Changed(ended)
    }
}

impl Machine for Computers {
    type Input = ComputersInput;
    type Effect = ComputersEffect;

    fn step(
        state: ComputersState,
        input: ComputersInput,
    ) -> (ComputersState, Vec<ComputersEffect>) {
        computers_step(state, input)
    }

    async fn run(&self, effect: ComputersEffect) -> Vec<ComputersInput> {
        let input = match effect {
            ComputersEffect::Tell(text) => {
                self.live.notice(Notice::Words(text));
                return Vec::new();
            }
            ComputersEffect::Reload => ComputersInput::Found(self.link.read().await),
            ComputersEffect::LoadCandidates => {
                ComputersInput::Candidates(self.link.candidates().await)
            }
            ComputersEffect::LoadServerModels => {
                ComputersInput::ServerModels(self.link.models().await)
            }
            ComputersEffect::Add(node) => self.changed(Change::Add(node)).await,
            ComputersEffect::Remove(id) => self.changed(Change::Remove(id)).await,
            ComputersEffect::Answer(node, answer) => {
                self.changed(Change::Answer(node, answer)).await
            }
            ComputersEffect::Forget(node) => self.changed(Change::Forget(node)).await,
            ComputersEffect::AddServer(draft) => {
                let ended = self
                    .link
                    .change(Change::Server(draft))
                    .await
                    .map_err(|failure| self.sentence_of(failure));
                ComputersInput::ServerAdded(ended)
            }
        };
        vec![input]
    }
}
