//! Following inferd: look for it when the picker appears and again each time its bus name
//! changes hands, never starting it.

use super::bus::BusSource;
use super::io::{connect, next_change, next_owner, open, owner_changes, read};
use detent_model::{Absence, Described};
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyPath;
use zbus::Connection;
use zbus::fdo::NameOwnerChangedStream;

/// What the window learns about inferd, in the order it happens.
pub(crate) enum Followed {
    /// inferd is on the bus: its client and its rows.
    Opened(LiveClient, Described),
    /// inferd cannot be used, and why.
    Gone(Absence),
    /// inferd put a row's word in force.
    Changed(KeyPath, String),
}

async fn attach(connection: &Connection) -> Result<(LiveClient, Described), Absence> {
    let client = open(connection).await?;
    let described = read(&client).await?;
    Ok((client, described))
}

/// Tells `tell` what inferd is doing until the bus goes away. A daemon that is not running is
/// reported at once and looked for again when its name appears; one that stops is reported gone.
pub(crate) async fn follow(bus: &BusSource, mut tell: impl FnMut(Followed)) {
    let connection = match connect(bus).await {
        Ok(connection) => connection,
        Err(absence) => return tell(Followed::Gone(absence)),
    };
    let mut owners = match owner_changes(&connection).await {
        Ok(owners) => owners,
        Err(absence) => return tell(Followed::Gone(absence)),
    };
    loop {
        let still_on_bus = match attach(&connection).await {
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

/// Passes on each change inferd announces until its name changes hands (`Some`) or the bus
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
                Some((path, word)) => tell(Followed::Changed(path, word)),
                None => return next_owner(owners).await,
            },
        }
    }
}
