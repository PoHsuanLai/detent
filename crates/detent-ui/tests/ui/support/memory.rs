//! A fake memory (`org.quire.Memory1.Control`) that answers the loose-settings calls from the
//! settings file in the scratch config home, the way the real one reads it, and writes down what
//! it was asked to confirm. It knows two keys and the shipped value of each; a value is loose when
//! it is a different word, or a number below the shipped one.
#![allow(dead_code)]

use super::bus::PrivateBus;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// `(key, shipped value)` of the settings the fake can be asked about.
const SHIPPED: &[(&str, &str)] = &[
    ("memory.files.at_rest", "sealed"),
    ("memory.retention.search_days", "30"),
];

#[derive(Default)]
struct State {
    confirmed: BTreeMap<String, String>,
    /// What each `ConfirmSettings` was given, in order: `key=value` pairs joined by a comma.
    confirms: Vec<String>,
    reads: usize,
    /// The name of the refusal the next confirmation answers with.
    refuse: Option<&'static str>,
}

/// An error with one of memory's names.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.quire.Memory1.Error")]
enum Refusal {
    #[zbus(error)]
    ZBus(zbus::Error),
    NotAllowed(String),
    Invalid(String),
}

struct Control {
    file: PathBuf,
    state: Arc<Mutex<State>>,
}

fn is_loose(shipped: &str, value: &str) -> bool {
    match shipped.parse::<i64>() {
        Ok(days) => value.parse::<i64>().is_ok_and(|held| held < days),
        Err(_) => value != shipped,
    }
}

impl Control {
    /// The value the file holds for the dotted `key`, as text.
    fn held(&self, key: &str) -> Option<String> {
        let text = std::fs::read_to_string(&self.file).ok()?;
        let table: toml::Table = text.parse().ok()?;
        let mut at = &toml::Value::Table(table);
        for part in key.split('.') {
            at = at.get(part)?;
        }
        Some(match at {
            toml::Value::String(word) => word.clone(),
            other => other.to_string(),
        })
    }
}

#[zbus::interface(name = "org.quire.Memory1.Control")]
impl Control {
    async fn pending_settings(&self) -> Vec<(String, String, String)> {
        let mut state = self.state.lock().expect("state");
        state.reads += 1;
        SHIPPED
            .iter()
            .filter_map(|(key, shipped)| {
                let held = self.held(key)?;
                let waits = is_loose(shipped, &held)
                    && state.confirmed.get(*key).map(String::as_str) != Some(held.as_str());
                waits.then(|| ((*key).to_owned(), held, (*shipped).to_owned()))
            })
            .collect()
    }

    async fn confirm_settings(&self, values: HashMap<String, String>) -> Result<(), Refusal> {
        let mut state = self.state.lock().expect("state");
        match state.refuse.take() {
            Some("NotAllowed") => return Err(Refusal::NotAllowed(String::new())),
            Some(_) => return Err(Refusal::Invalid("not a setting".to_owned())),
            None => {}
        }
        let known = |key: &String| SHIPPED.iter().any(|(shipped, _)| shipped == key);
        if !values.keys().all(known) {
            return Err(Refusal::Invalid("not a setting".to_owned()));
        }
        let mut sorted: Vec<_> = values.into_iter().collect();
        sorted.sort();
        let line: Vec<String> = sorted.iter().map(|(k, v)| format!("{k}={v}")).collect();
        state.confirms.push(line.join(","));
        state.confirmed.extend(sorted);
        Ok(())
    }
}

/// The fake, kept alive while the test runs.
pub struct Fake {
    state: Arc<Mutex<State>>,
    _connection: zbus::Connection,
}

impl Fake {
    /// The next confirmation is refused with the refusal named `name` (`NotAllowed`, `Invalid`).
    pub fn refuse_next(&self, name: &'static str) {
        self.state.lock().expect("state").refuse = Some(name);
    }

    /// What was confirmed, in order, one entry per call: `key=value` pairs joined by a comma.
    pub fn confirms(&self) -> Vec<String> {
        self.state.lock().expect("state").confirms.clone()
    }

    /// How many times the waiting settings were read.
    pub fn reads(&self) -> usize {
        self.state.lock().expect("state").reads
    }
}

/// Serves memory, reading the settings file at `file`.
pub async fn serve(bus: &PrivateBus, file: PathBuf) -> Fake {
    let state: Arc<Mutex<State>> = Arc::default();
    let connection = bus.connect().await;
    connection
        .object_server()
        .at(
            "/org/quire/Memory1",
            Control {
                file,
                state: state.clone(),
            },
        )
        .await
        .expect("memory served");
    connection
        .request_name("org.quire.Memory1")
        .await
        .expect("memory's name");
    Fake {
        state,
        _connection: connection,
    }
}
