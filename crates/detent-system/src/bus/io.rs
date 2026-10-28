//! Talking to a daemon's settings module: connect, find the daemon, read its rows, follow it.

use super::daemon::Daemon;
use super::source::BusSource;
use super::vouch::Vouch;
use crate::deadline::{Deadline, peer};
use detent_callers::{RoleCheck, desktop_peers};
use detent_model::Absence;
use ds_peer::within;
use ds_settings::live::{Changes, LiveClient, LiveError, OwnerCheck, SenderName, Verdict};
use ds_settings::schema::KeyPath;
use porter_dbus::{BusFailure, classify};
use zbus::Connection;
use zbus::fdo::{DBusProxy, NameOwnerChangedStream};
use zbus::names::BusName;

/// A failure the person sees only as "failed": its cause goes to the log.
fn failed(why: impl std::fmt::Display) -> Absence {
    eprintln!("detent: a settings service failed: {why}");
    Absence::Failed
}

pub fn absence_of(error: LiveError) -> Absence {
    match error {
        LiveError::NotPermitted(_) => Absence::NotPermitted,
        LiveError::NotTheService(_) => Absence::Unavailable,
        other => failed(other),
    }
}

/// Why a call to a service's own interface failed: nobody on its name means it is not running, the
/// bus's refusal means this caller is not permitted, anything else is a failure.
pub fn absence_of_bus(error: &zbus::Error) -> Absence {
    match classify(error) {
        BusFailure::NoDaemon => Absence::NotRunning,
        BusFailure::Denied(_) => Absence::NotPermitted,
        _ => failed(error),
    }
}

/// What a live module's refusal says, in the module's own words and without the name of the
/// error: that name is for a program, and a toast or a pane shows what the person reads.
pub fn plain(error: &LiveError) -> String {
    match error {
        LiveError::NotPermitted(words)
        | LiveError::UnknownKey(words)
        | LiveError::BadValue(words)
        | LiveError::Unavailable(words)
        | LiveError::Failed(words)
        | LiveError::NotTheService(words)
        | LiveError::Timeout(words) => words.clone(),
        _ => "The service did not answer as expected.".to_owned(),
    }
}

/// A connection to the bus. A bus that does not answer inside the read deadline (a socket that
/// accepts and says nothing) is a failure, not a window that never opens.
pub async fn connect(bus: &BusSource) -> Result<Connection, Absence> {
    within(Deadline::READ, peer::DESKTOP, dial(bus))
        .await
        .map_err(failed)?
}

async fn dial(bus: &BusSource) -> Result<Connection, Absence> {
    match bus {
        BusSource::Absent => Err(Absence::NoBus),
        BusSource::Session => Connection::session().await.map_err(failed),
        BusSource::Address(address, _) => zbus::connection::Builder::address(address.as_str())
            .map_err(failed)?
            .build()
            .await
            .map_err(failed),
    }
}

/// Whether `name` has an owner on the bus now: a look at the bus, never a start.
async fn name_owned(connection: &Connection, name: &str) -> Result<bool, Absence> {
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
pub async fn owner_changes(
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
pub async fn next_owner(owners: &mut NameOwnerChangedStream) -> Option<()> {
    use zbus::export::futures_core::Stream;
    std::future::poll_fn(|cx| std::pin::Pin::new(&mut *owners).poll_next(cx))
        .await
        .map(|_| ())
}

/// The check the owners of service names on `bus` are held to: the desktop's caller table on the
/// session bus, the check a test named on a bus it made.
fn vouch_for(bus: &BusSource, connection: &Connection) -> Vouch {
    match bus {
        BusSource::Address(_, vouch) => vouch.clone(),
        BusSource::Session | BusSource::Absent => {
            Vouch::of(RoleCheck::new(desktop_peers(connection)))
        }
    }
}

/// The unique name that owns `name`, none when nobody does.
async fn owner_of(connection: &Connection, name: &str) -> Result<Option<SenderName>, Absence> {
    let name = BusName::try_from(name).map_err(failed)?;
    let bus = DBusProxy::new(connection).await.map_err(failed)?;
    match bus.get_name_owner(name).await {
        Ok(owner) => Ok(Some(SenderName(owner.as_str().to_owned()))),
        Err(zbus::fdo::Error::NameHasNoOwner(_)) => Ok(None),
        Err(other) => Err(failed(other)),
    }
}

/// The owner of the service `name`, once it holds the role of that service in the desktop's
/// caller table. `NotRunning` when nobody owns the name, `Unavailable` when its owner is not the
/// service (a program that took the name first, or one that could not be told apart from it): a
/// caller then sends it nothing and shows nothing from it.
pub async fn confirmed(
    bus: &BusSource,
    connection: &Connection,
    name: &str,
) -> Result<SenderName, Absence> {
    let looked = async {
        let Some(owner) = owner_of(connection, name).await? else {
            return Err(Absence::NotRunning);
        };
        match vouch_for(bus, connection).holds_role(name, &owner).await {
            Verdict::Allow => Ok(owner),
            Verdict::Refuse => Err(Absence::Unavailable),
        }
    };
    within(Deadline::READ, peer::DESKTOP, looked)
        .await
        .map_err(failed)?
}

/// A client of the daemon's module, only when the daemon is already on the bus: asking for the
/// module of a daemon that is not running would start it, and opening Settings must not. The
/// daemons a page reads are started when the page is shown (`activate::start`), by name, so
/// this stays a look. The client asks the owner check before its first read and again whenever
/// the name changes hands, so a program that merely took the name is `Unavailable` and is sent
/// nothing, a typed secret included.
pub async fn open(
    bus: &BusSource,
    connection: &Connection,
    daemon: &Daemon,
) -> Result<LiveClient, Absence> {
    if !name_owned(connection, daemon.name).await? {
        return Err(Absence::NotRunning);
    }
    LiveClient::new(
        connection,
        daemon.name,
        &daemon.path,
        vouch_for(bus, connection),
    )
    .map_err(absence_of)
}

/// The next change the module announces, as a row and its value.
pub async fn next_change(changes: &mut Changes) -> Option<(KeyPath, toml::Value)> {
    use zbus::export::futures_core::Stream;
    loop {
        let item =
            std::future::poll_fn(|cx| std::pin::Pin::new(&mut *changes).poll_next(cx)).await?;
        if let Ok(change) = item {
            return Some((change.key, change.value));
        }
    }
}
