//! What the sync service says of a dataset (`Sync1.Status`) and the two lines under a synced
//! service: when it last synced, and how much of the account's storage it uses. Pure; the window
//! reads the service and hands the numbers in.

use super::hold::{DatasetName, segment_of};
use super::model::{Account, AccountKey, AccountService, AccountsPane};
use crate::engine_install::size_words;
use crate::updates::elapsed_words;
use std::collections::BTreeMap;

/// The services whose data the sync service keeps whole under the service's own name: their
/// dataset is `<account>/<kind>`.
const SYNCED_KINDS: [&str; 3] = ["calendar", "contacts", "tasks"];

/// Whether the person stopped a dataset from syncing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPause {
    /// They did.
    Paused,
    /// They did not.
    Running,
}

/// How full the account's storage is, as the provider reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageQuota {
    /// Bytes in use.
    pub used: u64,
    /// Bytes allowed; none when the provider reports no limit.
    pub total: Option<u64>,
}

impl StorageQuota {
    /// "2.1 GB of 15 GB", or "2.1 GB used" when there is no limit.
    pub fn line(&self) -> String {
        match self.total {
            Some(total) => format!("{} of {}", storage_words(self.used), storage_words(total)),
            None => format!("{} used", storage_words(self.used)),
        }
    }
}

/// A size in gigabytes with a whole number read plainly: "15 GB", not "15.0 GB".
fn storage_words(bytes: u64) -> String {
    size_words(bytes).replace(".0 ", " ")
}

/// Whether the sync service is moving data for a dataset now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncWork {
    /// It has items left to send or fetch.
    Working,
    /// It is waiting, paused or caught up.
    Resting,
}

/// What `Sync1.Status` answers for one dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncStatus {
    /// Seconds since the dataset last synced; none before its first cycle.
    pub anchor_age: Option<i64>,
    /// Items with work left.
    pub pending: u64,
    /// Conflicts nobody has settled.
    pub conflicts: u64,
    /// Whether the person paused it.
    pub pause: SyncPause,
    /// The storage the account has used, when the provider says.
    pub quota: Option<StorageQuota>,
}

impl SyncStatus {
    /// Whether data is moving now: a paused dataset moves nothing.
    pub fn work(&self) -> SyncWork {
        match self.pause {
            SyncPause::Running if self.pending > 0 => SyncWork::Working,
            SyncPause::Running | SyncPause::Paused => SyncWork::Resting,
        }
    }

    /// The line under the service: "Synced 5 minutes ago", "Syncing 3 items", "Paused".
    pub fn sync_line(&self) -> String {
        match (self.pause, self.anchor_age) {
            (SyncPause::Paused, _) => "Paused".to_owned(),
            _ if self.work() == SyncWork::Working => format!(
                "Syncing {} {}",
                self.pending,
                if self.pending == 1 { "item" } else { "items" }
            ),
            (SyncPause::Running, Some(age)) => format!("Synced {}", elapsed_words(age)),
            (SyncPause::Running, None) => "Waiting to sync".to_owned(),
        }
    }

    /// Both lines in one, as the row's second line: "Synced 5 minutes ago. 2.1 GB of 15 GB".
    pub fn lines(&self) -> String {
        match &self.quota {
            Some(quota) => format!("{}. {}", self.sync_line(), quota.line()),
            None => self.sync_line(),
        }
    }
}

/// What the sync service said of each dataset the window asked about.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncBook(BTreeMap<DatasetName, SyncStatus>);

impl SyncBook {
    /// A book of these answers.
    pub fn of(answers: impl IntoIterator<Item = (DatasetName, SyncStatus)>) -> SyncBook {
        SyncBook(answers.into_iter().collect())
    }

    /// What the service said of `dataset`.
    pub fn status_of(&self, dataset: &DatasetName) -> Option<&SyncStatus> {
        self.0.get(dataset)
    }

    /// Whether the service is moving data for any dataset of `account` now.
    pub fn work_of(&self, account: &Account) -> SyncWork {
        let busy = account
            .datasets()
            .iter()
            .filter_map(|dataset| self.status_of(dataset))
            .any(|status| status.work() == SyncWork::Working);
        if busy {
            SyncWork::Working
        } else {
            SyncWork::Resting
        }
    }
}

impl AccountService {
    /// The dataset the sync service keeps for this service of `account`; none for a service the
    /// sync service does not mirror.
    pub fn dataset(&self, account: &AccountKey) -> Option<DatasetName> {
        SYNCED_KINDS
            .contains(&self.kind.as_str())
            .then(|| DatasetName(format!("{}/{}", segment_of(account), self.kind)))
    }
}

impl Account {
    /// The datasets the sync service runs for the account: one per sync switch that is on, and
    /// one per mirrored service that is on.
    pub fn datasets(&self) -> Vec<DatasetName> {
        let switches = self
            .sync
            .iter()
            .filter(|sync| sync.on)
            .map(|sync| DatasetName::of_switch(&self.key, &sync.path));
        let services = self
            .services
            .iter()
            .filter(|service| service.on)
            .filter_map(|service| service.dataset(&self.key));
        switches.chain(services).collect()
    }
}

impl AccountsPane {
    /// The datasets of every account, to ask the sync service about.
    pub fn datasets(&self) -> Vec<DatasetName> {
        self.accounts.iter().flat_map(Account::datasets).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{StorageQuota, SyncPause, SyncStatus, SyncWork};

    fn status(
        anchor_age: Option<i64>,
        pending: u64,
        pause: SyncPause,
        quota: Option<StorageQuota>,
    ) -> SyncStatus {
        SyncStatus {
            anchor_age,
            pending,
            conflicts: 0,
            pause,
            quota,
        }
    }

    #[test]
    fn a_dataset_says_when_it_synced_how_full_the_storage_is_and_whether_it_is_working() {
        let some = |used, total| Some(StorageQuota { used, total });
        let cases = [
            (
                status(
                    Some(300),
                    0,
                    SyncPause::Running,
                    some(2_100_000_000, Some(15_000_000_000)),
                ),
                "Synced 5 minutes ago. 2.1 GB of 15 GB",
                SyncWork::Resting,
            ),
            (
                status(Some(10), 0, SyncPause::Running, some(500_000_000, None)),
                "Synced just now. 500 MB used",
                SyncWork::Resting,
            ),
            (
                status(Some(90_000), 0, SyncPause::Running, None),
                "Synced yesterday",
                SyncWork::Resting,
            ),
            (
                status(None, 0, SyncPause::Running, None),
                "Waiting to sync",
                SyncWork::Resting,
            ),
            (
                status(Some(60), 3, SyncPause::Running, None),
                "Syncing 3 items",
                SyncWork::Working,
            ),
            (
                status(Some(60), 1, SyncPause::Running, None),
                "Syncing 1 item",
                SyncWork::Working,
            ),
            (
                status(Some(60), 3, SyncPause::Paused, None),
                "Paused",
                SyncWork::Resting,
            ),
        ];
        for (held, lines, work) in cases {
            assert_eq!(held.lines(), lines, "{held:?}");
            assert_eq!(held.work(), work, "{held:?}");
        }
    }
}
