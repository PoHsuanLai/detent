//! A fake inferd: `org.quire.SettingsModule1` at inferd's path on a private bus, with the rows
//! the real module describes (`ai.model.<kind>.<tier>`, each a Menu of "", `auto` and models).
#![allow(dead_code)]

use super::bus::PrivateBus;
use ds_settings::live::{Access, Caller, LiveError, LiveModule, LiveSchema, Verdict, serve};
use ds_settings::schema::{
    AgentSetting, ChoiceGroup, ChoiceWord, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page,
    Section, UnavailableReason, WordLabels,
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

/// The rows of an inferd that has moved to slots (`ai.model.<slot>.<tier>`): the slot's key
/// segment and the display name its section carries (`Models / Text`).
const SLOT_ROWS: &[(&str, &str, &str)] = &[
    ("text", "Text", "fast"),
    ("text", "Text", "balanced"),
    ("text", "Text", "best"),
    ("voice_in", "Speech to text", "fast"),
    ("computer_use", "Computer use", "balanced"),
];

/// The display word of a tier's key segment.
fn tier_word(tier: &str) -> &'static str {
    match tier {
        "fast" => "Fast",
        "balanced" => "Balanced",
        _ => "Demanding",
    }
}

/// The words a slot's rows list, and the group each model sits in (slots layout only).
const WORDS: [(&str, &str, Option<&str>); 6] = [
    ("", "Catalogue default", None),
    ("auto", "Automatic", None),
    ("local/qwen", "qwen", Some("On this computer")),
    ("anthropic/claude", "claude", Some("Anthropic")),
    ("google/gemini", "gemini", Some("Google")),
    ("local/holo", "holo", Some("On this computer")),
];

/// Which keys the fake serves.
#[derive(Clone, Copy)]
pub enum Layout {
    /// The old kind segments (`llm`, `computer_use`).
    Kinds,
    /// The slot segments.
    Slots,
}

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
    /// Words no row can be set to, each with the reason: `unavailable` on every row.
    pub gated: Arc<Mutex<BTreeMap<String, String>>>,
    policy: Policy,
    layout: Layout,
}

impl Inferd {
    pub fn new(policy: Policy) -> Inferd {
        Inferd {
            values: Arc::default(),
            sets: Arc::default(),
            release: Arc::default(),
            held: Arc::default(),
            gated: Arc::default(),
            policy,
            layout: Layout::Kinds,
        }
    }

    /// The same fake serving slot keys.
    pub fn on_slots(mut self) -> Inferd {
        self.layout = Layout::Slots;
        self
    }

    /// Serves the module on `bus`, owning inferd's bus name; the connection must be kept alive.
    pub async fn serve_on(&self, bus: &PrivateBus) -> zbus::Connection {
        let connection = bus.connect().await;
        // The module first, the name after: a client told the name is taken finds the module.
        serve(&connection, INFERENCE_SETTINGS_PATH, self.clone())
            .await
            .expect("serves");
        connection.request_name(INFERENCE_BUS).await.expect("name");
        connection
    }

    /// One row. The old layout sends section `Models`, label `Language: Fast` and no groups;
    /// the slots layout sends section `Models / Text`, label `Fast` and the groups.
    fn spec(&self, kind: &str, slot: &str, tier: &str) -> KeySpec {
        let slots = matches!(self.layout, Layout::Slots);
        let words = match slots {
            true => WORDS.to_vec(),
            false => WORDS[..2]
                .iter()
                .chain(&WORDS[2..3])
                .chain(&WORDS[5..])
                .copied()
                .collect(),
        };
        let (label, section) = match slots {
            true => (tier_word(tier).to_owned(), format!("Models / {slot}")),
            false => (format!("{slot}: {}", tier_word(tier)), "Models".to_owned()),
        };
        KeySpec {
            path: KeyPath(format!("ai.model.{kind}.{tier}")),
            kind: KeyKind::Menu {
                variants: words
                    .iter()
                    .map(|(word, _, _)| (*word).to_owned())
                    .collect(),
            },
            default: toml::Value::String(String::new()),
            label: Label(label),
            help: Help(format!("The {slot} model at this tier.")),
            page: Page::Intelligence,
            section: Section(section),
            exposure: Exposure::Basic,
            labels: WordLabels(
                words
                    .iter()
                    .map(|(word, text, _)| ((*word).to_owned(), (*text).to_owned()))
                    .collect(),
            ),
            agent: AgentSetting::HandsOff,
            unavailable: self
                .gated
                .lock()
                .expect("gated")
                .iter()
                .map(|(word, why)| (ChoiceWord(word.clone()), UnavailableReason(why.clone())))
                .collect(),
            groups: match slots {
                true => words
                    .iter()
                    .filter_map(|(word, _, group)| {
                        group.map(|group| {
                            (
                                ChoiceWord((*word).to_owned()),
                                ChoiceGroup(group.to_owned()),
                            )
                        })
                    })
                    .collect(),
                false => BTreeMap::new(),
            },
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
            key: match self.layout {
                Layout::Kinds => ROWS,
                Layout::Slots => SLOT_ROWS,
            }
            .iter()
            .map(|(kind, label, tier)| self.spec(kind, label, tier))
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
