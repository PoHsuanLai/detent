//! A fake Spaces registry (`org.quire.Spaces1` on accountd's name) and a fake memory
//! (`org.quire.Memory1.Control`) on a private bus, writing what they were asked, in order, into
//! one log.
#![allow(dead_code)]

use super::bus::PrivateBus;
use porter_dbus::names::{SPACE_KEY_CREATED, SPACE_KEY_LOOK, SPACE_KEY_NAME};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::zvariant::{OwnedValue, Value};

type Details = HashMap<String, OwnedValue>;

/// What the fakes were asked, in order: `memory <space> <removal json>` and `registry remove <space>`.
pub type Log = Arc<Mutex<Vec<String>>>;

struct Registry {
    rows: Arc<Mutex<Vec<(String, String)>>>,
    log: Log,
}

fn owned(value: Value<'_>) -> OwnedValue {
    OwnedValue::try_from(value).expect("a plain value")
}

#[zbus::interface(name = "org.quire.Spaces1")]
impl Registry {
    async fn list(&self) -> Vec<(String, Details)> {
        self.rows
            .lock()
            .expect("rows")
            .iter()
            .map(|(id, name)| {
                let details: Details = [
                    (SPACE_KEY_NAME.to_owned(), owned(Value::from(name.as_str()))),
                    (SPACE_KEY_LOOK.to_owned(), owned(Value::from("{}"))),
                    (SPACE_KEY_CREATED.to_owned(), owned(Value::from(1_i64))),
                ]
                .into_iter()
                .collect();
                (id.clone(), details)
            })
            .collect()
    }

    async fn remove(&self, id: String) -> zbus::fdo::Result<()> {
        self.log
            .lock()
            .expect("log")
            .push(format!("registry remove {id}"));
        self.rows
            .lock()
            .expect("rows")
            .retain(|(row, _)| *row != id);
        Ok(())
    }
}

struct Control {
    log: Log,
}

#[zbus::interface(name = "org.quire.Memory1.Control")]
impl Control {
    async fn remove_space(&self, space: String, removal: String) -> String {
        self.log
            .lock()
            .expect("log")
            .push(format!("memory {space} {removal}"));
        r#"{"moved":0,"kept_pending":0,"deleted":0,"events_moved":0,"events_deleted":0}"#.to_owned()
    }
}

/// The fakes, kept alive while the test runs.
pub struct Fakes {
    pub log: Log,
    _registry: zbus::Connection,
    _memory: Option<zbus::Connection>,
}

/// Serves the registry holding `spaces` (`(id, name)`), and memory unless `memory` is false.
pub async fn serve(bus: &PrivateBus, spaces: &[(&str, &str)], memory: bool) -> Fakes {
    let log: Log = Arc::default();
    let rows = Arc::new(Mutex::new(
        spaces
            .iter()
            .map(|(id, name)| ((*id).to_owned(), (*name).to_owned()))
            .collect::<Vec<_>>(),
    ));
    let registry = bus.connect().await;
    registry
        .object_server()
        .at(
            "/org/quire/Spaces1",
            Registry {
                rows,
                log: log.clone(),
            },
        )
        .await
        .expect("registry served");
    registry
        .request_name("org.quire.Accounts1")
        .await
        .expect("accountd's name");
    let memory = if memory {
        let connection = bus.connect().await;
        connection
            .object_server()
            .at("/org/quire/Memory1", Control { log: log.clone() })
            .await
            .expect("memory served");
        connection
            .request_name("org.quire.Memory1")
            .await
            .expect("memory's name");
        Some(connection)
    } else {
        None
    };
    Fakes {
        log,
        _registry: registry,
        _memory: memory,
    }
}
