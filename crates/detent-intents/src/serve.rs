//! Putting the provider on the bus.

use crate::application::APP_NAME;
use crate::application::export_application;
use crate::context::SettingsContext;
use crate::provider::SettingsProvider;
use docket_client::{TransportError, serve_on, serve_on_with};
use docket_dbus::BusConnection;
use docket_dbus::callers::Callers;
use ds_peer::{Deadline, within};
use std::sync::Arc;

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

/// [`serve`], answering the process `callers` names as intentd instead of intentd's unit on the
/// system's `/proc`: a test's stand-in for the router.
pub async fn serve_with<K: Callers>(
    connection: &BusConnection,
    provider: SettingsProvider,
    callers: Arc<K>,
) -> Result<(), TransportError> {
    export_application(connection, provider.window()).await?;
    let context = SettingsContext(provider.app().clone());
    serve_on_with(connection, provider, context.clone(), context, callers).await
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
///
/// A bus that does not answer inside the read deadline is [`ServeFault::Connect`]: Settings then
/// opens without the service instead of never opening (audit DETENT-07).
pub async fn serve_session(provider: SettingsProvider) -> Result<BusConnection, ServeFault> {
    within(BUS_DEADLINE, BUS_PEER, serve_unbounded(provider))
        .await
        .map_err(|silent| ServeFault::Connect(silent.to_string()))?
}

/// How long the session bus gets to take the service: the read deadline, since a healthy bus
/// answers in milliseconds and a loaded login can take a few seconds.
const BUS_DEADLINE: Deadline = Deadline::READ;

/// Who is not answering, as a sentence names them.
pub(crate) const BUS_PEER: &str = "The desktop";

async fn serve_unbounded(provider: SettingsProvider) -> Result<BusConnection, ServeFault> {
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
