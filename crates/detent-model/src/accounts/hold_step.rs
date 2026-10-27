//! The held deletions' one transition: `(holds, input) -> (holds, effects)`.

use super::hold::{DatasetName, HoldCounts, SyncHold};
use super::model::Account;
use ds_settings::schema::KeyPath;

/// The most deletions kept at once. A person syncs a handful of datasets; a sync service (or a
/// program posing as one) that tells more is not believed, so the banners cannot grow with its
/// signals. A hold already kept is still updated or ended.
pub const HOLDS_MAX: usize = 64;

/// The deletions waiting for the person, kept apart from the accounts' rows: they come from the
/// sync service, not from the accounts module, and may be told before the rows are read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Holds(pub Vec<SyncHold>);

impl Holds {
    /// The holds on the dataset the sync switch at `row` of `account` controls.
    pub fn on_switch<'a>(
        &'a self,
        account: &'a Account,
        row: &'a KeyPath,
    ) -> impl Iterator<Item = &'a SyncHold> {
        self.0
            .iter()
            .filter(move |hold| hold.belongs_to(&account.key) && hold.is_on_switch(row))
    }

    /// The holds on datasets of `account` that none of its sync switches controls.
    pub fn on_no_switch<'a>(&'a self, account: &'a Account) -> impl Iterator<Item = &'a SyncHold> {
        self.0.iter().filter(move |hold| {
            hold.belongs_to(&account.key)
                && !account
                    .sync
                    .iter()
                    .any(|sync| hold.is_on_switch(&sync.path))
        })
    }

    /// Whether `account` has a hold on any of its datasets.
    pub fn any_of(&self, account: &Account) -> bool {
        self.0.iter().any(|hold| hold.belongs_to(&account.key))
    }
}

/// What happened to the holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoldInput {
    /// The sync service told a dataset's hold: new counts, or `None` when the hold ended.
    Told {
        /// The dataset.
        dataset: DatasetName,
        /// What is held now.
        counts: Option<HoldCounts>,
    },
    /// The person pressed Keep.
    Keep(DatasetName),
    /// The person pressed Remove.
    Remove(DatasetName),
    /// The sync service said nothing is held on the dataset (it was answered a moment ago).
    NothingHeld(DatasetName),
    /// The sync service refused or could not be asked.
    Refused(DatasetName),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoldEffect {
    /// Call `ConfirmDiscard(dataset)`.
    Confirm(DatasetName),
    /// Ask the sync service for the dataset's state again and tell it as `Told`.
    Refresh(DatasetName),
    /// Tell the person the files could not be removed.
    CouldNotRemove,
}

/// The holds after `input`, and what to do about it.
pub fn hold_step(holds: Holds, input: HoldInput) -> (Holds, Vec<HoldEffect>) {
    let Holds(mut list) = holds;
    let mut effects = Vec::new();
    match input {
        HoldInput::Told { dataset, counts } => {
            list.retain(|hold| hold.dataset != dataset);
            if let Some(counts) = counts.filter(|_| list.len() < HOLDS_MAX) {
                list.push(SyncHold { dataset, counts });
            }
        }
        // Keep asks nothing of the service: the files stay, and the service asks again by itself
        // if they are still gone. The banner only gets out of the way.
        HoldInput::Keep(dataset) => list.retain(|hold| hold.dataset != dataset),
        HoldInput::Remove(dataset) => {
            if list.iter().any(|hold| hold.dataset == dataset) {
                effects.push(HoldEffect::Confirm(dataset));
            }
        }
        HoldInput::NothingHeld(dataset) => effects.push(HoldEffect::Refresh(dataset)),
        HoldInput::Refused(_) => effects.push(HoldEffect::CouldNotRemove),
    }
    (Holds(list), effects)
}
