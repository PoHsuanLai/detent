//! Talking to a daemon's settings module: connect, find the daemon, read its rows, follow it.

use super::bus::BusSource;
use super::daemon::Daemon;
use crate::deadline::{Deadline, peer};
use detent_model::Absence;
use ds_peer::within;
use ds_settings::live::{Changes, LiveClient, LiveError};
use ds_settings::schema::KeyPath;
use zbus::Connection;
use zbus::fdo::{DBusProxy, NameOwnerChangedStream};
use zbus::names::BusName;

fn failed(why: impl ToString) -> Absence {
    Absence::Failed(why.to_string())
}

pub(super) fn absence_of(error: LiveError) -> Absence {
    match error {
        LiveError::NotPermitted(_) => Absence::NotPermitted,
        LiveError::Timeout(words) => Absence::Failed(words),
        other => failed(plain(&other)),
    }
}

/// What a live module's refusal says, in the module's own words and without the name of the
/// error: that name is for a program, and a toast or a pane shows what the person reads.
pub(crate) fn plain(error: &LiveError) -> String {
    match error {
        LiveError::NotPermitted(words)
        | LiveError::UnknownKey(words)
        | LiveError::BadValue(words)
        | LiveError::Unavailable(words)
        | LiveError::Failed(words)
        | LiveError::Timeout(words) => words.clone(),
        _ => "The service did not answer as expected.".to_owned(),
    }
}

/// A connection to the bus. A bus that does not answer inside the read deadline (a socket that
/// accepts and says nothing) is a failure, not a window that never opens.
pub(crate) async fn connect(bus: &BusSource) -> Result<Connection, Absence> {
    within(Deadline::READ, peer::DESKTOP, dial(bus))
        .await
        .map_err(failed)?
}

async fn dial(bus: &BusSource) -> Result<Connection, Absence> {
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

/// Whether `name` has an owner on the bus now: a look at the bus, never a start.
pub(crate) async fn name_owned(connection: &Connection, name: &str) -> Result<bool, Absence> {
    let name = BusName::try_from(name).map_err(failed)?;
    let asked = async {
        let bus = DBusProxy::new(connection).await.map_err(failed)?;
        bus.name_has_owner(name).await.map_err(failed)
    };
    within(Deadline::READ, peer::DESKTOP, asked)
        .await
        .map_err(failed)?
}

/// The signals that say the daemon's bus name changed hands, subscribed before anyone asks
/// whether it is there, so no change falls between the question and the listening.
pub(crate) async fn owner_changes(
    connection: &Connection,
    daemon: &Daemon,
) -> Result<NameOwnerChangedStream, Absence> {
    let subscribed = async {
        let bus = DBusProxy::new(connection).await.map_err(failed)?;
        bus.receive_name_owner_changed_with_args(&[(0, daemon.name)])
            .await
            .map_err(failed)
    };
    within(Deadline::READ, peer::DESKTOP, subscribed)
        .await
        .map_err(failed)?
}

/// The next time the daemon's name changes hands; `None` when the bus is gone.
pub(crate) async fn next_owner(owners: &mut NameOwnerChangedStream) -> Option<()> {
    use zbus::export::futures_core::Stream;
    std::future::poll_fn(|cx| std::pin::Pin::new(&mut *owners).poll_next(cx))
        .await
        .map(|_| ())
}

/// A client of the daemon's module, only when the daemon is already on the bus: asking for the
/// module of a daemon that is not running would start it, and opening Settings must not. The
/// daemons a page reads are started when the page is shown (`activate::start`), by name, so
/// this stays a look.
pub(super) async fn open(connection: &Connection, daemon: &Daemon) -> Result<LiveClient, Absence> {
    if !name_owned(connection, daemon.name).await? {
        return Err(Absence::NotRunning);
    }
    LiveClient::new(connection, daemon.name, &daemon.path)
        .await
        .map_err(absence_of)
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
