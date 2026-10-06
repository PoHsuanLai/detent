//! A fake inferd: `org.quire.SettingsModule1` at inferd's path on a private bus, with the rows
//! the real module describes (`ai.model.<kind>.<tier>`, each a Menu of "", `auto` and models).
#![allow(dead_code)]

use super::bus::PrivateBus;
use ds_settings::live::{Access, Caller, LiveError, LiveModule, LiveSchema, Verdict, serve};
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, WordLabels,
};
use porter_dbus::{INFERENCE_BUS, INFERENCE_SETTINGS_PATH};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// The tiers each fake kind lists.
const ROWS: &[(&str, &str, &str)] = &[
    ("llm", "Language", "fast"),
    ("llm", "Language", "balanced"),
    ("llm", "Language", "best"),
    ("computer_use", "Computer use", "balanced"),
];

/// What the fake does with a caller.
#[derive(Clone, Copy)]
pub enum Policy {
    /// Everyone may.
    Allow,
    /// Nobody may: inferd without the Settings caller row.
    Refuse,
}

/// The fake's state, shared with the test that watches it.
#[derive(Clone)]
pub struct Inferd {
    pub values: Arc<Mutex<BTreeMap<String, String>>>,
    /// Every `Set` that arrived, in order.
    pub sets: Arc<Mutex<Vec<(String, String)>>>,
    /// A `Set` waits here before it answers, when `held` is set.
    pub release: Arc<Notify>,
    pub held: Arc<Mutex<bool>>,
    policy: Policy,
}

impl Inferd {
    pub fn new(policy: Policy) -> Inferd {
        Inferd {
            values: Arc::default(),
            sets: Arc::default(),
            release: Arc::default(),
            held: Arc::default(),
            policy,
        }
    }

    /// Serves the module on `bus`, owning inferd's bus name; the connection must be kept alive.
    pub async fn serve_on(&self, bus: &PrivateBus) -> zbus::Connection {
        let connection = bus.connect().await;
        connection.request_name(INFERENCE_BUS).await.expect("name");
        serve(&connection, INFERENCE_SETTINGS_PATH, self.clone())
            .await
            .expect("serves");
        connection
    }

    fn spec(kind: &str, label: &str, tier: &str) -> KeySpec {
        let words = ["", "auto", "local/qwen", "local/holo"];
        KeySpec {
            path: KeyPath(format!("ai.model.{kind}.{tier}")),
            kind: KeyKind::Menu {
                variants: words.map(str::to_owned).to_vec(),
            },
            default: toml::Value::String(String::new()),
            label: Label(format!("{label}: {tier}")),
            help: Help(String::new()),
            page: Page::Intelligence,
            section: Section("Models".to_owned()),
            exposure: Exposure::Basic,
            labels: WordLabels(
                [
                    ("", "Catalogue default"),
                    ("auto", "Automatic"),
                    ("local/qwen", "qwen"),
                    ("local/holo", "holo"),
                ]
                .map(|(word, text)| (word.to_owned(), text.to_owned()))
                .into(),
            ),
            agent: AgentSetting::HandsOff,
        }
    }
}

impl LiveModule for Inferd {
    async fn permit(&self, _caller: &Caller, _access: Access) -> Verdict {
        match self.policy {
            Policy::Allow => Verdict::Allow,
            Policy::Refuse => Verdict::Refuse,
        }
    }

    async fn describe(&self) -> LiveSchema {
        LiveSchema {
            version: 1,
            key: ROWS
                .iter()
                .map(|(kind, label, tier)| Inferd::spec(kind, label, tier))
                .collect(),
        }
    }

    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        let values = self.values.lock().expect("values");
        Ok(toml::Value::String(
            values.get(&key.0).cloned().unwrap_or_default(),
        ))
    }

    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        let toml::Value::String(word) = value else {
            return Err(LiveError::BadValue("a model is text".to_owned()));
        };
        self.sets
            .lock()
            .expect("sets")
            .push((key.0.clone(), word.clone()));
        let held = *self.held.lock().expect("held");
        if held {
            self.release.notified().await;
        }
        self.values
            .lock()
            .expect("values")
            .insert(key.0.clone(), word);
        Ok(())
    }
}
