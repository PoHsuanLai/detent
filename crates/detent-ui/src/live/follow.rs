//! Following a daemon: look for it when its pane appears and again each time its bus name
//! changes hands, never starting it.

use super::bus::BusSource;
use super::daemon::Daemon;
use super::io::{connect, next_change, next_owner, open, owner_changes, read};
use detent_model::{Absence, Described};
use ds_settings::live::LiveClient;
use ds_settings::schema::{KeyKind, KeyPath};
use zbus::Connection;
use zbus::fdo::NameOwnerChangedStream;

/// What the window learns about the daemon, in the order it happens.
pub(crate) enum Followed {
    /// The daemon is on the bus: its client and its rows.
    Opened(LiveClient, Described),
    /// The daemon cannot be used, and why.
    Gone(Absence),
    /// The daemon put a row's value in force.
    Changed(KeyPath, toml::Value),
}

async fn attach(
    connection: &Connection,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
) -> Result<(LiveClient, Described), Absence> {
    let client = open(connection, daemon).await?;
    let described = read(&client, valued).await?;
    Ok((client, described))
}

/// `daemon`'s module and its rows once, when the daemon is already on the bus: for a caller that
/// asks a question, maybe sets a row, and follows nothing.
pub(crate) async fn open_once(
    bus: &BusSource,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
) -> Result<(LiveClient, Described), Absence> {
    let connection = connect(bus).await?;
    attach(&connection, daemon, valued).await
}

/// The rows of `daemon`'s module once, when the daemon is already on the bus: for a caller that
/// asks one question and follows nothing.
pub(crate) async fn read_once(
    bus: &BusSource,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
) -> Result<Described, Absence> {
    open_once(bus, daemon, valued)
        .await
        .map(|(_, described)| described)
}

/// The rows of `client` as they stand now, a value for each row `valued` picks out.
pub(crate) async fn reread(
    client: &LiveClient,
    valued: fn(&KeyKind) -> bool,
) -> Result<Described, Absence> {
    read(client, valued).await
}

/// Tells `tell` what `daemon` is doing until the bus goes away; `valued` picks the rows whose
/// value is read with the rows. A daemon that is not running is reported at once and looked for
/// again when its name appears; one that stops is reported gone.
pub(crate) async fn follow(
    bus: &BusSource,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
    mut tell: impl FnMut(Followed),
) {
    let connection = match connect(bus).await {
        Ok(connection) => connection,
        Err(absence) => return tell(Followed::Gone(absence)),
    };
    let mut owners = match owner_changes(&connection, daemon).await {
        Ok(owners) => owners,
        Err(absence) => return tell(Followed::Gone(absence)),
    };
    loop {
        let still_on_bus = match attach(&connection, daemon, valued).await {
            Ok((client, described)) => {
                tell(Followed::Opened(client.clone(), described));
                relay_changes(&client, &mut owners, &mut tell).await
            }
            Err(absence) => {
                tell(Followed::Gone(absence));
                next_owner(&mut owners).await
            }
        };
        if still_on_bus.is_none() {
            return;
        }
    }
}

/// Passes on each change the daemon announces until its name changes hands (`Some`) or the bus
/// closes (`None`).
async fn relay_changes(
    client: &LiveClient,
    owners: &mut NameOwnerChangedStream,
    tell: &mut impl FnMut(Followed),
) -> Option<()> {
    let Ok(mut changes) = client.changes().await else {
        return next_owner(owners).await;
    };
    loop {
        tokio::select! {
            owner = next_owner(owners) => return owner,
            change = next_change(&mut changes) => match change {
                Some((path, value)) => tell(Followed::Changed(path, value)),
                None => return next_owner(owners).await,
            },
        }
    }
}
