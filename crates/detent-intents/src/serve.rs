//! Putting the provider on the bus.

use crate::application::APP_NAME;
use crate::application::export_application;
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
    // The activation object goes up before the name is claimed: a caller that was waiting for
    // the name finds it answering.
    export_application(connection, provider.window()).await?;
    let context = SettingsContext(provider.app().clone());
    serve_on(connection, provider, context.clone(), context).await
}

/// Why the session bus could not be served.
#[derive(Debug, thiserror::Error)]
pub enum ServeFault {
    /// There is no session bus to connect to.
    #[error("cannot connect to the session bus: {0}")]
    Connect(String),
    /// Another Settings already owns the name: the caller should hand over to it.
    #[error("org.quire.Settings is already running")]
    Running,
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
    if name_taken(&connection).await {
        return Err(ServeFault::Running);
    }
    serve(&connection, provider)
        .await
        .map_err(|e| ServeFault::Serve(e.to_string()))?;
    Ok(connection)
}

/// Whether another connection already owns `org.quire.Settings`.
async fn name_taken(connection: &BusConnection) -> bool {
    let Ok(proxy) = zbus::fdo::DBusProxy::new(connection).await else {
        return false;
    };
    let Ok(name) = zbus::names::BusName::try_from(APP_NAME) else {
        return false;
    };
    proxy.name_has_owner(name).await.unwrap_or(false)
}
