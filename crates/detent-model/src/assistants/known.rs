//! What the machine knows of places to run: the signed-in accounts of the Intelligence group.

use crate::accounts::{AccountStatus, AccountsPane, Place};
use docket_settings::{Known, KnownPlace, PlaceName};
use porter_core::sheet::ProviderGroup;

/// The cloud accounts accountd lists that are signed in: an account of the Intelligence group
/// that is not a program on this computer, and does not need signing in again. Each carries its
/// provider's name, which the confirmation says. No source lists a person's other computers, so
/// none is known here; the ones the setting holds are shown as they are.
pub fn known_of(pane: &AccountsPane) -> Known {
    Known(
        pane.in_group(ProviderGroup::Intelligence)
            .filter(|account| account.place == Place::Elsewhere)
            .filter(|account| matches!(account.status, AccountStatus::Ok | AccountStatus::Limited))
            .map(|account| {
                let provider = account
                    .provider
                    .as_ref()
                    .map_or_else(|| account.label.clone(), |provider| provider.name.clone());
                KnownPlace::Cloud(PlaceName(account.label.clone()), provider)
            })
            .collect(),
    )
}
