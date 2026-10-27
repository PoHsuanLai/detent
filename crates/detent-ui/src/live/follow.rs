//! Following a daemon: look for it when its pane appears and again each time its bus name
//! changes hands, never starting it (`detent_system::bus::start` does that, once per page visit).

use detent_model::{Absence, Described};
use detent_system::bus::{
    BusSource, Daemon, attach_listening, connect, next_change, next_owner, owner_changes,
};
use ds_settings::live::{Changes, LiveClient};
use ds_settings::schema::{KeyKind, KeyPath};
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
        let still_on_bus = match attach_listening(bus, &connection, daemon, valued).await {
            Ok((client, described, changes)) => {
                tell(Followed::Opened(client, described));
                relay_changes(changes, &mut owners, &mut tell).await
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
    mut changes: Changes,
    owners: &mut NameOwnerChangedStream,
    tell: &mut impl FnMut(Followed),
) -> Option<()> {
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
