//! The networks this computer keeps a password for: removing them.

use super::nm::SavedConnectionProxy;
use super::read::saved;
use crate::deadline::{Deadline, peer};
use crate::live::BusSource;
use crate::system::system_connection;
use detent_model::Ssid;
use ds_peer::within;
use zbus::Connection;
use zbus::zvariant::OwnedObjectPath;

const NOT_AVAILABLE: &str = "Wi-Fi isn\u{2019}t available right now. Try again in a moment.";

/// Deletes the saved connection at `path`; one that is already gone is as good.
pub(super) async fn delete_saved(connection: &Connection, path: &OwnedObjectPath) {
    let Ok(builder) = SavedConnectionProxy::builder(connection).path(path.clone()) else {
        return;
    };
    if let Ok(saved) = builder.build().await {
        let _ = saved.delete().await;
    }
}

/// Removes every kept connection to `ssid`, so the computer no longer joins it by itself.
pub(super) async fn forget(bus: &BusSource, ssid: Ssid) -> Result<(), String> {
    let connection = system_connection(bus)
        .await
        .map_err(|_| NOT_AVAILABLE.to_owned())?;
    let removing = async {
        for (path, kept) in saved(&connection).await {
            if kept == ssid {
                delete_saved(&connection, &path).await;
            }
        }
    };
    within(Deadline::ACTION, peer::NETWORK, removing)
        .await
        .map_err(|silent| format!("{silent}."))
}
