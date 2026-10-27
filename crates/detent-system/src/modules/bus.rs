//! The real link: the module is looked for on the bus, followed by its name's owner and by the
//! `Changed` signals it sends to this peer.

use super::{Followed, Module, ModulesLink};
use crate::bus::{
    BusSource, Daemon, attach_listening, connect, next_change, next_owner, owner_changes,
};
use detent_model::Work;
use ds_settings::live::Changes;
use ds_settings::schema::KeyKind;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use zbus::fdo::NameOwnerChangedStream;

/// The modules of the daemons on `bus`.
#[derive(Debug, Clone)]
pub struct OnBus(pub BusSource);

impl ModulesLink for OnBus {
    fn follow<'a>(
        &'a self,
        daemon: &'a Daemon,
        valued: fn(&KeyKind) -> bool,
        told: UnboundedSender<Followed>,
    ) -> Work<'a, ()> {
        Box::pin(follow(&self.0, daemon, valued, told))
    }
}

/// Tells `told` what `daemon` is doing until the bus goes away.
async fn follow(
    bus: &BusSource,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
    told: UnboundedSender<Followed>,
) {
    let connection = match connect(bus).await {
        Ok(connection) => connection,
        Err(absence) => return tell(&told, Followed::Gone(absence)),
    };
    let mut owners = match owner_changes(&connection, daemon).await {
        Ok(owners) => owners,
        Err(absence) => return tell(&told, Followed::Gone(absence)),
    };
    loop {
        let still_on_bus = match attach_listening(bus, &connection, daemon, valued).await {
            Ok((client, described, changes)) => {
                tell(
                    &told,
                    Followed::Opened(Module::new(Arc::new(client)), described),
                );
                relay_changes(changes, &mut owners, &told).await
            }
            Err(absence) => {
                tell(&told, Followed::Gone(absence));
                next_owner(&mut owners).await
            }
        };
        if still_on_bus.is_none() {
            return;
        }
    }
}

/// Passes `followed` on; nobody listening any more is the pane's end, not a failure.
fn tell(told: &UnboundedSender<Followed>, followed: Followed) {
    let _gone = told.send(followed);
}

/// Passes on each change the daemon announces until its name changes hands (`Some`) or the bus
/// closes (`None`).
async fn relay_changes(
    mut changes: Changes,
    owners: &mut NameOwnerChangedStream,
    told: &UnboundedSender<Followed>,
) -> Option<()> {
    loop {
        tokio::select! {
            owner = next_owner(owners) => return owner,
            change = next_change(&mut changes) => match change {
                Some((path, value)) => tell(told, Followed::Changed(path, value)),
                None => return next_owner(owners).await,
            },
        }
    }
}
