//! Putting the provider on the bus.

use crate::context::SettingsContext;
use crate::provider::SettingsProvider;
use docket_client::{TransportError, serve_on};
use docket_dbus::BusConnection;

/// Exports `provider` at `/org/quire/IntentProvider1` on `connection` and claims the bus name
/// `org.quire.Settings`. Returns once the name is ours; the connection keeps serving.
pub async fn serve(
    connection: &BusConnection,
    provider: SettingsProvider,
) -> Result<(), TransportError> {
    let context = SettingsContext(provider.app().clone());
    serve_on(connection, provider, context.clone(), context).await
}

/// Why the session bus could not be served.
#[derive(Debug, thiserror::Error)]
pub enum ServeFault {
    /// There is no session bus to connect to.
    #[error("cannot connect to the session bus: {0}")]
    Connect(String),
    /// The bus is there and the provider could not take its place on it (the name is taken, or
    /// the object could not be exported).
    #[error("cannot serve org.quire.Settings: {0}")]
    Serve(String),
}

/// Connects to the session bus (`DBUS_SESSION_BUS_ADDRESS`, read by the bus library) and
/// serves `provider` on it. The connection is returned: dropping it takes the service off the
/// bus, so the caller keeps it for as long as the app runs.
pub async fn serve_session(provider: SettingsProvider) -> Result<BusConnection, ServeFault> {
    let connection = zbus::connection::Builder::session()
        .map_err(|e| ServeFault::Connect(e.to_string()))?
        .build()
        .await
        .map_err(|e| ServeFault::Connect(e.to_string()))?;
    serve(&connection, provider)
        .await
        .map_err(|e| ServeFault::Serve(e.to_string()))?;
    Ok(connection)
}
