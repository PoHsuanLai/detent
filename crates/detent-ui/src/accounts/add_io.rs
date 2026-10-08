//! Opening the account service's add sheet from Settings, through porter's client: the Settings
//! role may call `AddAccount`.

use crate::live::{BusSource, connect};
use detent_model::{AccountKey, AddEnd};
use porter_client::{Accounts, ClientError, DbusTransport, TransportError};
use porter_core::wire::{ParentWindow, ProviderHint, Refusal};

/// The bus's own word for a second sheet of a kind from the same app.
const SHEET_OPEN: &str = "LimitsExceeded";

/// Asks for the add sheet and waits for it to end.
pub(super) async fn add_account(bus: &BusSource) -> AddEnd {
    let Ok(connection) = connect(bus).await else {
        return AddEnd::Failed;
    };
    let accounts = Accounts::over(DbusTransport::over(connection));
    match accounts
        .add_account(ProviderHint::Any, &ParentWindow::Unparented)
        .await
    {
        Ok(_) => AddEnd::Added,
        Err(error) => end_of(&error),
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
