//! "Sync now" on an account's sync row: which dataset the press names, and the line the person
//! reads for how the sync service took it.

use super::hold::DatasetName;
use super::model::AccountKey;
use ds_settings::schema::KeyPath;

/// How the sync service took the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncNowEnd {
    /// It will sync at once.
    Started,
    /// The person paused syncing of the dataset.
    Paused,
    /// It could not be asked, or did not take the request.
    Failed,
}

impl SyncNowEnd {
    /// The line the person reads.
    pub fn line(self) -> &'static str {
        match self {
            Self::Started => "Syncing now.",
            Self::Paused => "Syncing is paused for this account.",
            Self::Failed => "Could not start syncing. Try again later.",
        }
    }
}

impl DatasetName {
    /// The dataset the sync switch at `row` (`accounts.<id>.sync.<leaf>`) of `account` controls.
    pub fn of_switch(account: &AccountKey, row: &KeyPath) -> Self {
        let leaf = row.0.rsplit('.').next().unwrap_or_default();
        Self(format!("{}/{leaf}", super::hold::segment_of(account)))
    }
}
