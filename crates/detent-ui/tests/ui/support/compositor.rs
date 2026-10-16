//! A fake compositor: `org.quire.SettingsModule1` at `/org/quire/Compositor1/privacy` on
//! `org.quire.Compositor1` on a private bus, listing the Privacy rows the way the real one does
//! (`privacy.<power>.<id>` with `.forget` beside a person's answer, a read-out for an
//! administrator's, a line for a power nobody asked in) and writing down what it was asked.
#![allow(dead_code)]

use super::bus::PrivateBus;
use ds_settings::live::{
    Access, Caller, LiveError, LiveModule, LiveSchema, Served, Verdict, serve,
};
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Note, Page, Section,
};
use std::sync::{Arc, Mutex};

pub const BUS: &str = "org.quire.Compositor1";
pub const PATH: &str = "/org/quire/Compositor1/privacy";

/// The powers the compositor names in a row's path.
const POWERS: [&str; 7] = [
    "screen",
    "keys",
    "typing_help",
    "clipboard",
    "lock",
    "displays",
    "windows",
];

/// Who set a row.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Person,
    Administrator,
}

/// One answer the fake holds.
#[derive(Clone)]
pub struct Row {
    pub power: &'static str,
    pub id: &'static str,
    pub subject: &'static str,
    pub word: &'static str,
    pub source: Source,
}

impl Row {
    pub fn person(
        power: &'static str,
        id: &'static str,
        subject: &'static str,
        word: &'static str,
    ) -> Row {
        Row {
            power,
            id,
            subject,
            word,
            source: Source::Person,
        }
    }

    pub fn administrator(
        power: &'static str,
        id: &'static str,
        subject: &'static str,
        word: &'static str,
    ) -> Row {
        Row {
            power,
            id,
            subject,
            word,
            source: Source::Administrator,
        }
    }

    fn base(&self) -> String {
        format!("privacy.{}.{}", self.power, self.id)
    }
}

/// The fake's state, shared with the test that watches it.
#[derive(Clone, Default)]
pub struct Compositor {
    pub rows: Arc<Mutex<Vec<Row>>>,
    /// What it was asked, in order: `set <path> <word>` and `forget <path>`.
    pub log: Arc<Mutex<Vec<String>>>,
}

fn spec(path: String, label: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        path: KeyPath(path),
        kind,
        default: toml::Value::Boolean(false),
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Apps,
        section: Section("Screen".to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

impl Compositor {
    pub fn holding(rows: Vec<Row>) -> Compositor {
        Compositor {
            rows: Arc::new(Mutex::new(rows)),
            log: Arc::default(),
        }
    }

    pub fn log(&self) -> Vec<String> {
        self.log.lock().expect("log").clone()
    }

    /// Serves the module on `bus`, owning the compositor's name; keep both alive.
    pub async fn serve_on(&self, bus: &PrivateBus) -> (zbus::Connection, Served) {
        let connection = bus.connect().await;
        let served = serve(&connection, PATH, self.clone())
            .await
            .expect("serves");
        connection.request_name(BUS).await.expect("name");
        (connection, served)
    }

    fn keys(&self) -> Vec<KeySpec> {
        let rows = self.rows.lock().expect("rows").clone();
        POWERS
            .iter()
            .flat_map(|power| {
                let of_power: Vec<&Row> = rows.iter().filter(|row| row.power == *power).collect();
                if of_power.is_empty() {
                    return vec![spec(
                        format!("privacy.{power}.none"),
                        "No program has asked yet.",
                        KeyKind::Fixed {
                            variant: String::new(),
                        },
                    )];
                }
                of_power
                    .into_iter()
                    .flat_map(|row| match row.source {
                        Source::Person => vec![
                            spec(
                                row.base(),
                                row.subject,
                                KeyKind::Toggle {
                                    variants: ["on".to_owned(), "off".to_owned()],
                                },
                            ),
                            spec(
                                format!("{}.forget", row.base()),
                                "Ask Again",
                                KeyKind::Live {
                                    action: LiveAction {
                                        label: ActionLabel("Ask Again".to_owned()),
                                        weight: ActionWeight::Plain,
                                    },
                                },
                            ),
                        ],
                        Source::Administrator => {
                            let mut read_out = spec(
                                row.base(),
                                row.subject,
                                KeyKind::Fixed {
                                    variant: row.word.to_owned(),
                                },
                            );
                            read_out.note = Some(Note("Set by your administrator".to_owned()));
                            vec![read_out]
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

impl LiveModule for Compositor {
    async fn permit(&self, _caller: &Caller, _access: Access) -> Verdict {
        Verdict::Allow
    }

    async fn describe(&self) -> LiveSchema {
        LiveSchema {
            version: 1,
            key: self.keys(),
        }
    }

    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        let rows = self.rows.lock().expect("rows");
        match rows.iter().find(|row| row.base() == key.0) {
            Some(row) => Ok(toml::Value::String(row.word.to_owned())),
            None if key.0.ends_with(".forget") || key.0.ends_with(".none") => {
                Ok(toml::Value::Boolean(false))
            }
            None => Err(LiveError::UnknownKey(key.0.clone())),
        }
    }

    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        let mut rows = self.rows.lock().expect("rows");
        if let Some(base) = key.0.strip_suffix(".forget") {
            self.log
                .lock()
                .expect("log")
                .push(format!("forget {}", key.0));
            rows.retain(|row| row.base() != base);
            return Ok(());
        }
        let Some(row) = rows.iter_mut().find(|row| row.base() == key.0) else {
            return Err(LiveError::UnknownKey(key.0.clone()));
        };
        if row.source == Source::Administrator {
            return Err(LiveError::NotPermitted(
                "Your administrator set this, so it cannot be changed here.".to_owned(),
            ));
        }
        let word = value.as_str().unwrap_or_default().to_owned();
        self.log
            .lock()
            .expect("log")
            .push(format!("set {} {word}", key.0));
        row.word = if word == "on" { "on" } else { "off" };
        Ok(())
    }
}
