//! Starting the daemons a page reads, once, when the page is shown.

use super::daemon::Daemon;
use super::io::connect;
use super::source::BusSource;
use crate::deadline::{Deadline, answered, peer};
use detent_model::ModuleId;
use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::names::WellKnownName;

/// Asks the bus to start the daemon of each of `modules` (`StartServiceByName`), without waiting
/// for any of them to come up: the pane that reads a daemon follows its name and reads it when it
/// appears. A daemon that is already running is left alone. A name with no service file
/// (`ServiceUnknown`), a refusal or a daemon that fails to start changes nothing: the daemon stays
/// absent and its page says so, as before.
pub async fn start(bus: &BusSource, modules: &[ModuleId]) {
    if modules.is_empty() {
        return;
    }
    let Ok(connection) = connect(bus).await else {
        return;
    };
    for module in modules {
        let (connection, name) = (connection.clone(), Daemon::of(*module).name);
        tokio::spawn(async move { start_one(&connection, name).await });
    }
}

/// Asks the bus to start the service `name` and waits for its answer, which comes once the
/// service has taken its name. The answer is of no use: whether it came up is learned from who
/// owns the name. A bus that never replies ends the wait at the deadline.
pub async fn start_one(connection: &Connection, name: &str) {
    let Ok(proxy) = DBusProxy::new(connection).await else {
        return;
    };
    let Ok(name) = WellKnownName::try_from(name) else {
        return;
    };
    let _ = answered(
        Deadline::ACTION,
        peer::DESKTOP,
        proxy.start_service_by_name(name, 0),
    )
    .await;
}
