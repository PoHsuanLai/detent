//! Opening the add sheet from Settings, through porter's client: one sheet in the account service
//! adds everything, an identity or an AI key. The Settings role may call `AddAccount`. The sheet
//! is asked of the owner of the service's name that was confirmed, never of the name.

use crate::bus::{BusSource, pinned_starting};
use crate::deadline::{Deadline, answered, peer};
use detent_callers::ACCOUNTS_BUS;
use detent_model::{AccountKey, AddEnd};
use porter_client::AccountsError;
use porter_core::ProviderHint;
use porter_wire::{ParentWindow, Refusal};

/// Asks for the add sheet and waits for it to end.
pub async fn add_account(bus: &BusSource) -> AddEnd {
    let Ok(accountd) = pinned_starting(bus, ACCOUNTS_BUS).await else {
        return AddEnd::Failed;
    };
    let Ok(porter) = accountd.accounts() else {
        return AddEnd::Failed;
    };
    let accounts = porter.accounts();
    let asked = answered(
        Deadline::PROMPT,
        peer::ACCOUNTS,
        accounts.add(ProviderHint::Any, &ParentWindow::Unparented),
    )
    .await;
    match asked {
        Some(Ok(_)) => AddEnd::Added,
        Some(Err(error)) => end_of(&error),
        None => AddEnd::Failed,
    }
}

/// How a failed ask ended.
fn end_of(error: &AccountsError) -> AddEnd {
    match error {
        AccountsError::AlreadyAdded(id) => AddEnd::AlreadyThere(AccountKey(id.as_str().to_owned())),
        AccountsError::Refused(Refusal::Dismissed) => AddEnd::Closed,
        AccountsError::Busy => AddEnd::Busy,
        _ => AddEnd::Failed,
    }
}
