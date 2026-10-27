//! A held deletion: the sync service saw that files an account keeps on this computer are gone
//! on the server, and waits for the person before it removes them here too. What the pane knows
//! of one.

use super::model::AccountKey;
use ds_settings::schema::KeyPath;

/// A dataset's name as the sync service gives it: `<account>/<dataset>`, where the account is
/// the id as one bus-path segment (`.` and `-` read `_`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatasetName(pub String);

/// The longest dataset name taken from the sync service, in bytes. The names are an account's id
/// and a dataset's kind; a signal with more is not the service's, and is dropped.
pub const DATASET_NAME_MAX: usize = 256;

impl DatasetName {
    /// The name `text`, when it is no longer than [`DATASET_NAME_MAX`]: the one way a name the sync
    /// service sent becomes a dataset name.
    pub fn bounded(text: &str) -> Option<DatasetName> {
        (text.len() <= DATASET_NAME_MAX).then(|| DatasetName(text.to_owned()))
    }

    fn parts(&self) -> (&str, &str) {
        self.0.split_once('/').unwrap_or((self.0.as_str(), ""))
    }

    /// The account part: the id as a bus-path segment.
    fn segment(&self) -> &str {
        self.parts().0
    }

    /// The dataset part (`files`, `photos`, ...).
    fn leaf(&self) -> &str {
        self.parts().1
    }
}

/// How many of the held files are gone on the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldCounts {
    /// The files that would be removed here.
    pub discard: u64,
    /// The files the dataset holds in all.
    pub held: u64,
}

/// A deletion waiting for the person's word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncHold {
    /// The dataset it is on.
    pub dataset: DatasetName,
    /// How much of it.
    pub counts: HoldCounts,
}

impl SyncHold {
    /// Whether the hold is on a dataset of `account`.
    pub(super) fn belongs_to(&self, account: &AccountKey) -> bool {
        self.dataset.segment() == segment_of(account)
    }

    /// Whether the hold is on the dataset a sync switch at `row` (`...sync.files`) controls: its
    /// dataset is the switch's leaf or one that starts with it (`photos_originals`).
    pub(super) fn is_on_switch(&self, row: &KeyPath) -> bool {
        let leaf = self.dataset.leaf();
        row.0.rsplit('.').next().is_some_and(|switch| {
            leaf == switch
                || leaf
                    .strip_prefix(switch)
                    .is_some_and(|r| r.starts_with('_'))
        })
    }
}

/// The id as the sync service writes it in a dataset's name.
pub(super) fn segment_of(account: &AccountKey) -> String {
    account
        .id()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
