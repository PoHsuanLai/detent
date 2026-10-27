//! The sync service's side of the Accounts pane: what it says of a dataset, and the deletions it
//! holds for the person's word. Looked for, never started.

mod bus;
mod holds;
mod status;

pub use bus::OnBus;
pub use holds::follow_holds;

use detent_model::{DatasetName, HoldInput, SyncNowEnd, SyncStatus, Work};
use std::fmt::Debug;

/// What the Accounts pane asks of the sync service.
pub trait SyncLink: Debug + Send + Sync {
    /// `dataset`'s status now; none when the service is not there or does not know it.
    fn status(&self, dataset: DatasetName) -> Work<'_, Option<SyncStatus>>;
    /// Lets the discard through for `dataset`: nothing to tell when the service took the word, the
    /// reason otherwise.
    fn confirm_discard(&self, dataset: DatasetName) -> Work<'_, Option<HoldInput>>;
    /// `dataset`'s hold as the service says it now.
    fn hold_now(&self, dataset: DatasetName) -> Work<'_, HoldInput>;
    /// Has the service sync `dataset` at once; it answers when it took the request.
    fn sync_now(&self, dataset: DatasetName) -> Work<'_, SyncNowEnd>;
}
