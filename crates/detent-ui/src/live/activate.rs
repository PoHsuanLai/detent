//! Starting the daemons a page reads, once, when the page is shown.

use super::bus::BusSource;
use super::daemon::Daemon;
use super::io::connect;
use detent_model::ModuleId;
use zbus::fdo::DBusProxy;
use zbus::names::WellKnownName;

/// Asks the bus to start the daemon of each of `modules` (`StartServiceByName`), without waiting
/// for any of them to come up: the pane that reads a daemon follows its name and reads it when it
/// appears. A daemon that is already running is left alone. A name with no service file
/// (`ServiceUnknown`), a refusal or a daemon that fails to start changes nothing: the daemon stays
/// absent and its page says so, as before.
pub(crate) async fn start(bus: &BusSource, modules: &[ModuleId]) {
    if modules.is_empty() {
        return;
    }
    let Ok(connection) = connect(bus).await else {
        return;
    };
    let Ok(proxy) = DBusProxy::new(&connection).await else {
        return;
    };
    for module in modules {
        let Ok(name) = WellKnownName::try_from(Daemon::of(*module).name) else {
            continue;
        };
        let proxy = proxy.clone();
        tokio::spawn(async move {
            // The reply is of no use: whether the daemon came up is learned from its name.
            let _ = proxy.start_service_by_name(name, 0).await;
        });
    }
}
