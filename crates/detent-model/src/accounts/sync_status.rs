//! What the sync service says of a dataset (`Sync1.Status`): when it last synced, what is left to
//! do, and how much of the account's storage it uses. Pure; the window reads the service and hands
//! the numbers in.

use super::hold::{DatasetName, segment_of};
use super::model::{Account, AccountKey, AccountService, AccountsPane};
use std::collections::BTreeMap;

/// The services whose data the sync service keeps whole under the service's own name: their
/// dataset is `<account>/<kind>`.
const SYNCED_KINDS: [&str; 3] = ["calendar", "contacts", "tasks"];

/// How full the account's storage is, as the provider reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageQuota {
    /// Bytes in use.
    pub used: u64,
    /// Bytes allowed; none when the provider reports no limit.
    pub total: Option<u64>,
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
    pub paused: bool,
    /// The storage the account has used, when the provider says.
    pub quota: Option<StorageQuota>,
}

impl SyncStatus {
    /// Whether data is moving now: a paused dataset moves nothing.
    pub fn working(&self) -> bool {
        !self.paused && self.pending > 0
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
    pub fn syncing(&self, account: &Account) -> bool {
        account
            .datasets()
            .iter()
            .filter_map(|dataset| self.status_of(dataset))
            .any(SyncStatus::working)
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
            .filter(|sync| sync.on.is_on())
            .map(|sync| DatasetName::of_switch(&self.key, &sync.path));
        let services = self
            .services
            .iter()
            .filter(|service| service.on.is_on())
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
    use super::SyncStatus;

    fn status(pending: u64, paused: bool) -> SyncStatus {
        SyncStatus {
            anchor_age: Some(60),
            pending,
            conflicts: 0,
            paused,
            quota: None,
        }
    }

    #[test]
    fn a_dataset_is_working_only_while_it_has_items_left_and_is_not_paused() {
        let cases = [
            (status(0, false), false),
            (status(3, false), true),
            (status(1, false), true),
            (status(3, true), false),
        ];
        for (held, working) in cases {
            assert_eq!(held.working(), working, "{held:?}");
        }
    }
}
