//! A fake syncd: `org.quire.Sync1` at its path on a private bus, holding the deletions a test
//! sets. `Watch` tells the holds on now; `ConfirmDiscard` clears one and tells the end, or answers
//! `NothingHeld`; every call is recorded.
#![allow(dead_code)]

use super::bus::PrivateBus;
use porter_dbus::Details;
use porter_dbus::names::{SYNC_BUS, SYNC_PATH};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

/// The answers of `ConfirmDiscard` that are not success.
#[derive(zbus::DBusError, Debug)]
#[zbus(prefix = "org.quire.Sync1.Error")]
enum SyncFault {
    #[zbus(error)]
    ZBus(zbus::Error),
    NothingHeld,
}

/// What is held now, by dataset: files gone, files held.
type Held = Arc<Mutex<BTreeMap<String, (u64, u64)>>>;

fn counts(discard: u64, held: u64) -> Details {
    let mut details = Details::new();
    for (key, count) in [("discard", discard), ("held", held)] {
        details.insert(
            key.to_owned(),
            Value::from(count).try_to_owned().expect("a number"),
        );
    }
    details
}

/// The fake's state, shared with the test that watches it.
#[derive(Clone, Default)]
pub struct Syncd {
    held: Held,
    /// Every dataset `ConfirmDiscard` was called with, in order.
    pub confirmed: Arc<Mutex<Vec<String>>>,
    /// How many times `Watch` was called.
    pub watched: Arc<Mutex<u32>>,
}

impl Syncd {
    /// Serves `org.quire.Sync1` on `bus`; the connection must be kept alive.
    pub async fn serve_on(&self, bus: &PrivateBus) -> zbus::Connection {
        let connection = bus.connect().await;
        connection
            .object_server()
            .at(SYNC_PATH, self.clone())
            .await
            .expect("serves");
        connection.request_name(SYNC_BUS).await.expect("name");
        connection
    }

    /// A hold starts (or changes): held now, and told.
    pub async fn hold(
        &self,
        connection: &zbus::Connection,
        dataset: &str,
        discard: u64,
        held: u64,
    ) {
        self.held
            .lock()
            .expect("held")
            .insert(dataset.to_owned(), (discard, held));
        let emitter = SignalEmitter::new(connection, SYNC_PATH).expect("emitter");
        Syncd::needs_confirmation(&emitter, dataset, counts(discard, held))
            .await
            .expect("signal");
    }

    /// A hold ends without anyone confirming: the server listed its files again.
    pub async fn heal(&self, connection: &zbus::Connection, dataset: &str) {
        self.held.lock().expect("held").remove(dataset);
        let emitter = SignalEmitter::new(connection, SYNC_PATH).expect("emitter");
        Syncd::needs_confirmation(&emitter, dataset, Details::new())
            .await
            .expect("signal");
    }

    /// Forgets a hold without telling anyone: the window's picture of it is now stale.
    pub fn forget(&self, dataset: &str) {
        self.held.lock().expect("held").remove(dataset);
    }

    pub fn confirmed(&self) -> Vec<String> {
        self.confirmed.lock().expect("confirmed").clone()
    }
}

#[zbus::interface(name = "org.quire.Sync1")]
impl Syncd {
    async fn watch(&self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
        *self.watched.lock().expect("watched") += 1;
        let now: Vec<_> = self
            .held
            .lock()
            .expect("held")
            .clone()
            .into_iter()
            .collect();
        for (dataset, (discard, held)) in now {
            let _ = Syncd::needs_confirmation(&emitter, &dataset, counts(discard, held)).await;
        }
    }

    async fn status(&self, dataset: &str) -> Details {
        let mut details = Details::new();
        if let Some((discard, held)) = self.held.lock().expect("held").get(dataset) {
            let nested = Value::from(counts(*discard, *held));
            details.insert(
                "needs_confirmation".to_owned(),
                nested.try_to_owned().expect("nested"),
            );
        }
        details
    }

    async fn confirm_discard(
        &self,
        dataset: &str,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> Result<(), SyncFault> {
        self.confirmed
            .lock()
            .expect("confirmed")
            .push(dataset.to_owned());
        let had = self.held.lock().expect("held").remove(dataset);
        if had.is_none() {
            return Err(SyncFault::NothingHeld);
        }
        let _ = Syncd::needs_confirmation(&emitter, dataset, Details::new()).await;
        Ok(())
    }

    #[zbus(signal)]
    async fn needs_confirmation(
        emitter: &SignalEmitter<'_>,
        dataset: &str,
        held: Details,
    ) -> zbus::Result<()>;
}
