//! The sync service's side of the Accounts pane: the deletions it holds for the person's word, and
//! what it says of each dataset. Looked for, never started.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{DatasetName, HoldEffect, HoldInput, Holds, SyncBook, hold_step};
use detent_system::bus::BusSource;
use detent_system::sync::{SyncLink, follow_holds};
use std::sync::{Arc, Mutex, PoisonError};
use tokio::sync::mpsc::unbounded_channel;

/// The held deletions' handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct HeldDeletions {
    bus: BusSource,
    sync: Arc<dyn SyncLink>,
    live: Live<Holds>,
}

pane!(HeldDeletions, Holds, follow);

impl HeldDeletions {
    pub(crate) fn over(bus: BusSource, sync: Arc<dyn SyncLink>) -> HeldDeletions {
        HeldDeletions {
            bus,
            sync,
            live: Live::default(),
        }
    }

    /// Gives the holds `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: HoldInput) -> Pending {
        self.dispatch(input)
    }

    /// Takes each hold the service starts or ends, until the bus goes away.
    async fn follow(&self) {
        let (told, mut heard) = unbounded_channel();
        let following = follow_holds(&self.bus, move |input| {
            let _gone = told.send(input);
        });
        let handling = async {
            while let Some(input) = heard.recv().await {
                self.dispatch(input).await;
            }
        };
        tokio::join!(following, handling);
    }
}

impl Machine for HeldDeletions {
    type Input = HoldInput;
    type Effect = HoldEffect;

    fn step(state: Holds, input: HoldInput) -> (Holds, Vec<HoldEffect>) {
        hold_step(state, input)
    }

    async fn run(&self, effect: HoldEffect) -> Vec<HoldInput> {
        match effect {
            HoldEffect::Confirm(dataset) => self
                .sync
                .confirm_discard(dataset)
                .await
                .into_iter()
                .collect(),
            HoldEffect::Refresh(dataset) => vec![self.sync.hold_now(dataset).await],
            HoldEffect::CouldNotRemove => {
                self.live.notice(Notice::HoldNotRemoved);
                Vec::new()
            }
        }
    }
}

/// What the sync service says of the datasets of the accounts shown. A dataset the service does
/// not answer for has no entry, and nothing is said of it.
#[derive(Debug, Clone)]
pub struct Datasets {
    sync: Arc<dyn SyncLink>,
    live: Live<SyncBook>,
    shown: Arc<Mutex<Vec<DatasetName>>>,
}

impl Datasets {
    pub(crate) fn over(sync: Arc<dyn SyncLink>) -> Datasets {
        Datasets {
            sync,
            live: Live::default(),
            shown: Arc::default(),
        }
    }

    /// The book, as a live value.
    pub fn live(&self) -> &Live<SyncBook> {
        &self.live
    }

    /// Reads the status of `datasets`, the ones of the accounts now shown, and keeps them for the
    /// next time it is read again.
    pub async fn read(&self, datasets: Vec<DatasetName>) {
        *self.shown.lock().unwrap_or_else(PoisonError::into_inner) = datasets.clone();
        let mut answers = Vec::new();
        for dataset in datasets {
            if let Some(status) = self.sync.status(dataset.clone()).await {
                answers.push((dataset, status));
            }
        }
        self.live.update(|_| SyncBook::of(answers));
    }

    /// Asks the service to sync `dataset` now, tells the person how it went, and reads the
    /// datasets again.
    pub async fn sync_now(&self, dataset: DatasetName) {
        let end = self.sync.sync_now(dataset).await;
        self.live.notice(Notice::SyncNow(end));
        let shown = self
            .shown
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        self.read(shown).await;
    }
}
