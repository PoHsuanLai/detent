//! Meeting a daemon's settings module: connect, open its client and read its rows, once or with
//! the changes it announces heard from before the reading.

use super::daemon::Daemon;
use super::io::{absence_of, connect, open};
use super::read::read;
use super::source::BusSource;
use detent_model::{Absence, Described};
use ds_settings::live::{Changes, LiveClient};
use ds_settings::schema::KeyKind;
use zbus::Connection;

/// `daemon`'s module and its rows, when the daemon is already on the bus.
pub async fn attach(
    bus: &BusSource,
    connection: &Connection,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
) -> Result<(LiveClient, Described), Absence> {
    let client = open(bus, connection, daemon).await?;
    let described = read(&client, valued).await?;
    Ok((client, described))
}

/// Like [`attach`], listening for changes first, so none falls between the reading and the
/// listening: what changes during the reading is heard after it.
pub async fn attach_listening(
    bus: &BusSource,
    connection: &Connection,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
) -> Result<(LiveClient, Described, Changes), Absence> {
    let client = open(bus, connection, daemon).await?;
    let changes = client.changes().await.map_err(absence_of)?;
    let described = read(&client, valued).await?;
    Ok((client, described, changes))
}

/// `daemon`'s module and its rows once, when the daemon is already on the bus: for a caller that
/// asks a question, maybe sets a row, and follows nothing.
pub async fn open_once(
    bus: &BusSource,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
) -> Result<(LiveClient, Described), Absence> {
    let connection = connect(bus).await?;
    attach(bus, &connection, daemon, valued).await
}

/// The rows of `daemon`'s module once, when the daemon is already on the bus: for a caller that
/// asks one question and follows nothing.
pub async fn read_once(
    bus: &BusSource,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
) -> Result<Described, Absence> {
    open_once(bus, daemon, valued)
        .await
        .map(|(_, described)| described)
}
