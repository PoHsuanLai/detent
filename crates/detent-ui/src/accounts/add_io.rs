//! Opening an add sheet from Settings, through porter's client: an identity through the account
//! service, an AI key through the key service. The Settings role may call each `AddAccount`.

use crate::live::{BusSource, connect};
use detent_model::{AccountKey, AddEnd};
use porter_client::{Accounts, AiKeys, ClientError, DbusTransport, TransportError};
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

/// What adding an AI key asks: the key service's own add sheet, ended with how it went. The one
/// seam to porter's `AiKeys` client, so the call is written once.
trait KeySheet {
    async fn add(&self) -> AddEnd;
}

/// The key service over a connection to the bus.
struct OverBus(zbus::Connection);

impl KeySheet for OverBus {
    async fn add(&self) -> AddEnd {
        let Ok(keys) = AiKeys::connect(&self.0).await else {
            return AddEnd::Failed;
        };
        match keys
            .add_account(ProviderHint::Any, &ParentWindow::Unparented)
            .await
        {
            Ok(_) => AddEnd::Added,
            Err(_) => AddEnd::Failed,
        }
    }
}

/// Asks the key service for its add sheet and waits for it to end.
pub(super) async fn add_ai_key(bus: &BusSource) -> AddEnd {
    match connect(bus).await {
        Ok(connection) => OverBus(connection).add().await,
        Err(_) => AddEnd::Failed,
    }
}
