//! Talking to a daemon's settings module: connect, find the daemon, read its rows, follow it.

use super::bus::BusSource;
use super::daemon::Daemon;
use detent_model::{Absence, Described};
use ds_settings::live::{Changes, LiveClient, LiveError};
use ds_settings::schema::{KeyKind, KeyPath};
use std::collections::BTreeMap;
use zbus::Connection;
use zbus::fdo::{DBusProxy, NameOwnerChangedStream};
use zbus::names::BusName;

fn failed(why: impl ToString) -> Absence {
    Absence::Failed(why.to_string())
}

fn absence_of(error: LiveError) -> Absence {
    match error {
        LiveError::NotPermitted(_) => Absence::NotPermitted,
        other => failed(other),
    }
}

pub(super) async fn connect(bus: &BusSource) -> Result<Connection, Absence> {
    match bus {
        BusSource::Absent => Err(Absence::NoBus),
        BusSource::Session => Connection::session().await.map_err(failed),
        BusSource::Address(address) => zbus::connection::Builder::address(address.as_str())
            .map_err(failed)?
            .build()
            .await
            .map_err(failed),
    }
}

/// The signals that say the daemon's bus name changed hands, subscribed before anyone asks
/// whether it is there, so no change falls between the question and the listening.
pub(super) async fn owner_changes(
    connection: &Connection,
    daemon: &Daemon,
) -> Result<NameOwnerChangedStream, Absence> {
    DBusProxy::new(connection)
        .await
        .map_err(failed)?
        .receive_name_owner_changed_with_args(&[(0, daemon.name)])
        .await
        .map_err(failed)
}

/// The next time the daemon's name changes hands; `None` when the bus is gone.
pub(super) async fn next_owner(owners: &mut NameOwnerChangedStream) -> Option<()> {
    use zbus::export::futures_core::Stream;
    std::future::poll_fn(|cx| std::pin::Pin::new(&mut *owners).poll_next(cx))
        .await
        .map(|_| ())
}

/// A client of the daemon's module, only when the daemon is already on the bus: asking for the
/// module of a daemon that is not running would start it, and opening Settings must not.
pub(super) async fn open(connection: &Connection, daemon: &Daemon) -> Result<LiveClient, Absence> {
    let name = BusName::try_from(daemon.name).map_err(failed)?;
    let running = DBusProxy::new(connection)
        .await
        .map_err(failed)?
        .name_has_owner(name)
        .await
        .map_err(failed)?;
    if !running {
        return Err(Absence::NotRunning);
    }
    LiveClient::new(connection, daemon.name, &daemon.path)
        .await
        .map_err(absence_of)
}

/// The module's rows and the value of each row `valued` picks out.
pub(super) async fn read(
    client: &LiveClient,
    valued: fn(&KeyKind) -> bool,
) -> Result<Described, Absence> {
    let keys = client.describe().await.map_err(absence_of)?.key;
    let mut values = BTreeMap::new();
    for spec in keys.iter().filter(|spec| valued(&spec.kind)) {
        if let Some(word) = client.get(&spec.path).await.map_err(absence_of)?.as_str() {
            values.insert(spec.path.clone(), word.to_owned());
        }
    }
    Ok(Described { keys, values })
}

/// The next change the module announces, as a row and its value.
pub(super) async fn next_change(changes: &mut Changes) -> Option<(KeyPath, toml::Value)> {
    use zbus::export::futures_core::Stream;
    loop {
        let item =
            std::future::poll_fn(|cx| std::pin::Pin::new(&mut *changes).poll_next(cx)).await?;
        if let Ok(change) = item {
            return Some((change.key, change.value));
        }
    }
}
