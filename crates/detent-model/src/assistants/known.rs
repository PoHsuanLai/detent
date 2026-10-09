//! What the machine knows of places to run: the signed-in accounts of the Intelligence group.

use crate::accounts::{AccountStatus, AccountsPane, Place};
use crate::computers::ComputerRow;
use docket_settings::{Known, KnownPlace, PlaceName};
use porter_core::sheet::ProviderGroup;

/// The cloud accounts accountd lists that are signed in: an account of the Intelligence group
/// that is not a program on this computer, and does not need signing in again. Each carries its
/// provider's name, which the confirmation says. The person's other computers are the ones the
/// assistant's service lists (`computers`); the ones only the setting holds are not shown.
pub fn known_of(pane: &AccountsPane, computers: &[ComputerRow]) -> Known {
    let own = computers
        .iter()
        .map(|computer| KnownPlace::OwnComputer(PlaceName(computer.name.clone())));
    Known(
        own.chain(
            pane.in_group(ProviderGroup::Intelligence)
                .filter(|account| account.place == Place::Elsewhere)
                .filter(|account| {
                    matches!(account.status, AccountStatus::Ok | AccountStatus::Limited)
                })
                .map(|account| {
                    let provider = account
                        .provider
                        .as_ref()
                        .map_or_else(|| account.label.clone(), |provider| provider.name.clone());
                    KnownPlace::Cloud(PlaceName(account.label.clone()), provider)
                }),
        )
        .collect(),
    )
}
