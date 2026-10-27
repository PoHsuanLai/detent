//! Opening an add sheet from Settings, through porter's client: an identity through the account
//! service, an AI key through the key service. The Settings role may call each `AddAccount`. The
//! sheet is asked of the owner of the service's name that was confirmed, never of the name.

use crate::bus::{BusSource, Pinned, pinned_starting};
use crate::deadline::{Deadline, answered, peer};
use detent_callers::{ACCOUNTS_BUS, INFERENCE_BUS};
use detent_model::{AccountKey, AddEnd};
use porter_client::{Accounts, AiKeys, ClientError, TransportError};
use porter_core::wire::{ParentWindow, ProviderHint, Refusal};

/// The bus's own word for a second sheet of a kind from the same app.
const SHEET_OPEN: &str = "LimitsExceeded";

/// Asks for the add sheet and waits for it to end.
pub async fn add_account(bus: &BusSource) -> AddEnd {
    let Ok(accountd) = pinned_starting(bus, ACCOUNTS_BUS).await else {
        return AddEnd::Failed;
    };
    let Ok(transport) = accountd.accounts() else {
        return AddEnd::Failed;
    };
    let accounts = Accounts::over(transport);
    let asked = answered(
        Deadline::PROMPT,
        peer::ACCOUNTS,
        accounts.add_account(ProviderHint::Any, &ParentWindow::Unparented),
    )
    .await;
    match asked {
        Some(Ok(_)) => AddEnd::Added,
        Some(Err(error)) => end_of(&error),
        None => AddEnd::Failed,
    }
}

/// How a failed ask ended.
fn end_of(error: &ClientError) -> AddEnd {
    match error {
        ClientError::AlreadyAdded(id) => AddEnd::AlreadyThere(AccountKey(id.as_str().to_owned())),
        ClientError::Refused(Refusal::Dismissed) => AddEnd::Closed,
        ClientError::Transport(TransportError::Malformed(why)) if why.contains(SHEET_OPEN) => {
            AddEnd::Busy
        }
        _ => AddEnd::Failed,
    }
}

/// What adding an AI key asks: the key service's own add sheet, ended with how it went. The one
/// seam to porter's `AiKeys` client, so the call is written once.
trait KeySheet {
    async fn add(&self) -> AddEnd;
}

/// The key service, at the owner of its name that was confirmed.
struct OverBus(Pinned);

impl KeySheet for OverBus {
    async fn add(&self) -> AddEnd {
        let Ok(to) = self.0.at_owner() else {
            return AddEnd::Failed;
        };
        let Some(Ok(keys)) = answered(
            Deadline::READ,
            peer::ACCOUNTS,
            AiKeys::connect_at(&self.0.connection, &to),
        )
        .await
        else {
            return AddEnd::Failed;
        };
        let asked = answered(
            Deadline::PROMPT,
            peer::ACCOUNTS,
            keys.add_account(ProviderHint::Any, &ParentWindow::Unparented),
        )
        .await;
        match asked {
            Some(Ok(_)) => AddEnd::Added,
            Some(Err(_)) | None => AddEnd::Failed,
        }
    }
}

/// Asks the key service for its add sheet and waits for it to end.
pub async fn add_ai_key(bus: &BusSource) -> AddEnd {
    let Ok(inferd) = pinned_starting(bus, INFERENCE_BUS).await else {
        return AddEnd::Failed;
    };
    OverBus(inferd).add().await
}
