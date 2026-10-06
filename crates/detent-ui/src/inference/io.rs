//! Talking to inferd's settings module: connect, read the rows, set one, follow changes.

use super::bus::BusSource;
use detent_model::{Absence, Described};
use ds_settings::live::{Changes, LiveClient, LiveError};
use ds_settings::schema::{KeyKind, KeyPath};
use porter_dbus::{INFERENCE_BUS, INFERENCE_SETTINGS_PATH};
use std::collections::BTreeMap;
use zbus::Connection;
use zbus::fdo::DBusProxy;
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

async fn connect(bus: &BusSource) -> Result<Connection, Absence> {
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

/// A client of inferd's picker, only when inferd is already on the bus: asking for the module
/// of a daemon that is not running would start it, and opening Settings must not.
pub(crate) async fn open(bus: &BusSource) -> Result<LiveClient, Absence> {
    let connection = connect(bus).await?;
    let name = BusName::try_from(INFERENCE_BUS).map_err(failed)?;
    let running = DBusProxy::new(&connection)
        .await
        .map_err(failed)?
        .name_has_owner(name)
        .await
        .map_err(failed)?;
    if !running {
        return Err(Absence::NotRunning);
    }
    LiveClient::new(&connection, INFERENCE_BUS, INFERENCE_SETTINGS_PATH)
        .await
        .map_err(absence_of)
}

/// The module's rows and the value of each menu row.
pub(crate) async fn read(client: &LiveClient) -> Result<Described, Absence> {
    let keys = client.describe().await.map_err(absence_of)?.key;
    let mut values = BTreeMap::new();
    for spec in keys
        .iter()
        .filter(|spec| matches!(spec.kind, KeyKind::Menu { .. }))
    {
        if let Some(word) = client.get(&spec.path).await.map_err(absence_of)?.as_str() {
            values.insert(spec.path.clone(), word.to_owned());
        }
    }
    Ok(Described { keys, values })
}

/// Sets the row at `path` to `word`; the module answers when it has put it in force.
pub(crate) async fn set(client: &LiveClient, path: &KeyPath, word: &str) -> Result<(), String> {
    client
        .set(path, &toml::Value::String(word.to_owned()))
        .await
        .map_err(|error| error.to_string())
}

/// The next change the module announces, as a row and its word.
pub(crate) async fn next_change(changes: &mut Changes) -> Option<(KeyPath, String)> {
    use zbus::export::futures_core::Stream;
    loop {
        let item =
            std::future::poll_fn(|cx| std::pin::Pin::new(&mut *changes).poll_next(cx)).await?;
        if let Some(word) = item.ok().and_then(|change| {
            change
                .value
                .as_str()
                .map(|word| (change.key, word.to_owned()))
        }) {
            return Some(word);
        }
    }
}
