//! Following a daemon's live settings module (`org.quire.SettingsModule1`) as a pane does: look
//! for it when the pane appears, read its rows, and read again each time it comes back or says a
//! row changed. Looking never starts it (`Client::start` does that, once per page visit).

use detent_model::{Absence, Described};
use detent_system::bus::{
    BusSource, Daemon, attach_listening, connect, next_change, next_owner, owner_changes,
};
use ds_settings::live::{Changes, LiveClient};
use ds_settings::schema::{KeyKind, KeyPath};
use std::sync::{Arc, Mutex, PoisonError};
use tokio::sync::mpsc::unbounded_channel;
use zbus::fdo::NameOwnerChangedStream;

/// What the client learns about the daemon, in the order it happens.
pub(crate) enum Followed {
    /// The daemon is on the bus: its client and its rows.
    Opened(LiveClient, Described),
    /// The daemon cannot be used, and why.
    Gone(Absence),
    /// The daemon put a row's value in force.
    Changed(KeyPath, toml::Value),
}

/// The client of a module while it is on the bus.
#[derive(Debug, Clone, Default)]
pub(crate) struct Slot(Arc<Mutex<Option<LiveClient>>>);

impl Slot {
    /// The client now, if the module is there.
    pub(crate) fn get(&self) -> Option<LiveClient> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The module is there with this client, or is gone.
    pub(crate) fn put(&self, client: Option<LiveClient>) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = client;
    }
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

/// [`follow`], with each event handled by `handle` one after the other, so a handler that waits
/// on a service holds up the next event and never the daemon's own signals.
pub(crate) async fn follow_with(
    bus: &BusSource,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
    mut handle: impl AsyncFnMut(Followed),
) {
    let (told, mut heard) = unbounded_channel();
    let following = follow(bus, daemon, valued, move |followed| {
        let _gone = told.send(followed);
    });
    let handling = async {
        while let Some(followed) = heard.recv().await {
            handle(followed).await;
        }
    };
    tokio::join!(following, handling);
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
