//! The models a place offers: the words inferd's picker lists (`<account>/<model>`) that belong
//! to the accounts the place is made of.

use super::step::PlaceRef;
use crate::accounts::{AccountKey, AccountsPane};
use crate::picker::Ready;
use docket_settings::ModelChoice;

/// One model a place offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOffer {
    /// What is stored when it is picked.
    pub choice: ModelChoice,
    /// What a person reads.
    pub label: String,
}

/// The accounts a place is made of: the runtimes on this computer, or the cloud account of that
/// name. A computer of the person's has none listed.
fn accounts_of(place: &PlaceRef, pane: &AccountsPane) -> Vec<AccountKey> {
    match place {
        PlaceRef::ThisComputer => pane.local_runtimes().map(|a| a.key.clone()).collect(),
        PlaceRef::Cloud(name) => pane
            .accounts
            .iter()
            .filter(|account| account.label == name.0)
            .map(|account| account.key.clone())
            .collect(),
        PlaceRef::Own(_) => Vec::new(),
    }
}

/// The models `place` offers among the rows of `picker`, each once, in the picker's order.
pub fn offers_for(place: &PlaceRef, pane: &AccountsPane, picker: &Ready) -> Vec<ModelOffer> {
    let keys = accounts_of(place, pane);
    let mut offers: Vec<ModelOffer> = Vec::new();
    for row in picker.kinds.iter().flat_map(|kind| kind.rows.iter()) {
        for word in row.sections.iter().flat_map(|section| section.words.iter()) {
            let ours = word
                .split_once('/')
                .is_some_and(|(account, _)| keys.iter().any(|key| key.0 == account));
            if ours && !offers.iter().any(|offer| offer.choice.0 == *word) {
                offers.push(ModelOffer {
                    choice: ModelChoice(word.clone()),
                    label: row.label_of(word),
                });
            }
        }
    }
    offers
}
