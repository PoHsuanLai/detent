//! The sync service from memory: what it says of each dataset, and how its answers end.

use super::SyncLink;
use crate::lock::locked;
use detent_model::{DatasetName, HoldInput, SyncNowEnd, SyncStatus, Work};
use std::sync::{Arc, Mutex};

/// What was asked of the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAsked {
    /// A hold's discard was let through.
    ConfirmDiscard(DatasetName),
    /// A syncing at once.
    SyncNow(DatasetName),
}

#[derive(Debug)]
struct Book {
    statuses: Vec<(DatasetName, SyncStatus)>,
    now: SyncNowEnd,
    asked: Vec<SyncAsked>,
}

/// A service that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeSync(Arc<Mutex<Book>>);

impl FakeSync {
    /// A service that knows these datasets; "Sync now" starts.
    pub fn knowing(statuses: Vec<(DatasetName, SyncStatus)>) -> FakeSync {
        FakeSync(Arc::new(Mutex::new(Book {
            statuses,
            now: SyncNowEnd::Started,
            asked: Vec::new(),
        })))
    }

    /// "Sync now" ends like this from now on.
    pub fn sync_now_ends(&self, end: SyncNowEnd) {
        locked(&self.0).now = end;
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<SyncAsked> {
        locked(&self.0).asked.clone()
    }
}

impl SyncLink for FakeSync {
    fn status(&self, dataset: DatasetName) -> Work<'_, Option<SyncStatus>> {
        let found = locked(&self.0)
            .statuses
            .iter()
            .find(|(name, _)| *name == dataset)
            .map(|(_, status)| status.clone());
        Box::pin(std::future::ready(found))
    }

    fn confirm_discard(&self, dataset: DatasetName) -> Work<'_, Option<HoldInput>> {
        locked(&self.0)
            .asked
            .push(SyncAsked::ConfirmDiscard(dataset));
        Box::pin(std::future::ready(None))
    }

    fn hold_now(&self, dataset: DatasetName) -> Work<'_, HoldInput> {
        Box::pin(std::future::ready(HoldInput::Told {
            dataset,
            counts: None,
        }))
    }

    fn sync_now(&self, dataset: DatasetName) -> Work<'_, SyncNowEnd> {
        let mut book = locked(&self.0);
        book.asked.push(SyncAsked::SyncNow(dataset));
        Box::pin(std::future::ready(book.now))
    }
}
