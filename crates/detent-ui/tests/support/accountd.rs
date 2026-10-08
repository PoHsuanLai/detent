//! A fake accountd: `org.quire.SettingsModule1` at accountd's path on a private bus, with the
//! rows the real module describes (`accounts.<id>.service.<kind>`, `.grant.<grant>`, `.sync.<leaf>`, `.reauth`,
//! `.remove`, `accounts.clients.<issuer>`). A removed account and a revoked grant leave the rows.
#![allow(dead_code)]

use super::bus::PrivateBus;
use ds_settings::live::{
    Access, Caller, LiveError, LiveModule, LiveSchema, Served, Verdict, serve,
};
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};
use porter_dbus::{ACCOUNTS_BUS, ACCOUNTS_SETTINGS_PATH};
use std::sync::{Arc, Mutex};

/// What the fake does with a caller.
#[derive(Clone, Copy)]
pub enum Policy {
    /// Everyone may.
    Allow,
    /// Nobody may: accountd without the Settings caller row.
    Refuse,
}

/// One account the fake holds.
#[derive(Clone)]
pub struct Held {
    pub id: String,
    pub label: String,
    pub state: String,
    pub services: Vec<(String, bool)>,
    pub grants: Vec<(String, String)>,
    /// The sync toggles (`files`, `photos`) with their state: only for an account syncd mirrors.
    pub sync: Vec<(String, bool)>,
    /// Whether the account is a runtime on this computer: the `place` row says so.
    pub on_this_computer: bool,
    /// The words of the read-only `expires` row, for a sign-in that expires.
    pub expires: Option<String>,
    /// The slug of the read-only `sign_in` row, when the module publishes one.
    pub sign_in: Option<String>,
}

impl Held {
    pub fn new(id: &str, label: &str, state: &str, services: &[&str]) -> Held {
        Held {
            id: id.to_owned(),
            label: label.to_owned(),
            state: state.to_owned(),
            services: services.iter().map(|s| ((*s).to_owned(), true)).collect(),
            grants: Vec::new(),
            sync: Vec::new(),
            on_this_computer: false,
            expires: None,
            sign_in: None,
        }
    }

    /// A language-model runtime on this computer: probed, no sign-in.
    pub fn runtime(id: &str, label: &str, state: &str) -> Held {
        Held {
            on_this_computer: true,
            ..Held::new(id, label, state, &["llm"])
        }
    }

    /// An account syncd can mirror: both sync toggles, off.
    pub fn syncing(mut self) -> Held {
        self.sync = vec![("files".to_owned(), false), ("photos".to_owned(), false)];
        self
    }

    /// A sign-in that expires: the `expires` row says `words`.
    pub fn expiring(mut self, words: &str) -> Held {
        self.expires = Some(words.to_owned());
        self
    }

    /// How the account signs in again: the `sign_in` row says `slug`.
    pub fn signing_in(mut self, slug: &str) -> Held {
        self.sign_in = Some(slug.to_owned());
        self
    }

    pub fn granting(mut self, grant: &str, what: &str) -> Held {
        self.grants.push((grant.to_owned(), what.to_owned()));
        self
    }
}

/// The fake's state, shared with the test that watches it.
#[derive(Clone)]
pub struct Accountd {
    pub accounts: Arc<Mutex<Vec<Held>>>,
    /// The Microsoft client id the fake holds.
    pub client: Arc<Mutex<String>>,
    /// Every `Set` that arrived, in order.
    pub sets: Arc<Mutex<Vec<(String, toml::Value)>>>,
    policy: Policy,
}

impl Accountd {
    pub fn new(policy: Policy, accounts: Vec<Held>) -> Accountd {
        Accountd {
            accounts: Arc::new(Mutex::new(accounts)),
            client: Arc::default(),
            sets: Arc::default(),
            policy,
        }
    }

    /// Serves the module on `bus`, owning accountd's bus name; the connection must be kept alive.
    pub async fn serve_on(&self, bus: &PrivateBus) -> zbus::Connection {
        self.serve_announcing(bus).await.0
    }

    /// The same, with the means to announce a change the module did not get through `Set`.
    pub async fn serve_announcing(&self, bus: &PrivateBus) -> (zbus::Connection, Served) {
        let connection = bus.connect().await;
        let served = serve(&connection, ACCOUNTS_SETTINGS_PATH, self.clone())
            .await
            .expect("serves");
        connection.request_name(ACCOUNTS_BUS).await.expect("name");
        (connection, served)
    }

    /// The account `id` is now in `state`; nobody is told until the daemon announces it.
    pub fn stop(&self, id: &str, state: &str) {
        for held in self.accounts.lock().expect("accounts").iter_mut() {
            if held.id == id {
                held.state = state.to_owned();
            }
        }
    }

    pub fn sets(&self) -> Vec<(String, toml::Value)> {
        self.sets.lock().expect("sets").clone()
    }
}

fn spec(path: String, section: &str, label: &str, kind: KeyKind, default: toml::Value) -> KeySpec {
    KeySpec {
        path: KeyPath(path),
        kind,
        default,
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Accounts,
        section: Section(section.to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        agent: AgentSetting::HandsOff,
    }
}

fn action(name: &str, weight: ActionWeight) -> KeyKind {
    KeyKind::Live {
        action: LiveAction {
            label: ActionLabel(name.to_owned()),
            weight,
        },
    }
}

fn rows_of(held: &Held) -> Vec<KeySpec> {
    let id = &held.id;
    let section = held.label.as_str();
    let fixed = |leaf: &str, label: &str, word: &str| {
        spec(
            format!("accounts.{id}.{leaf}"),
            section,
            label,
            KeyKind::Fixed {
                variant: word.to_owned(),
            },
            toml::Value::String(word.to_owned()),
        )
    };
    let mut rows = vec![
        fixed("label", "Account", &held.label),
        fixed("state", "Status", &held.state),
    ];
    if let Some(words) = &held.expires {
        let mut row = fixed("expires", "Sign-in", words);
        row.help = Help(
            "Google ends the sign-ins of an app that is not verified yet after 7 days; signing in again starts another 7."
                .to_owned(),
        );
        rows.push(row);
    }
    if let Some(slug) = &held.sign_in {
        rows.push(fixed("sign_in", "How", slug));
    }
    if held.on_this_computer {
        rows.push(fixed("place", "Where", "this_computer"));
    }
    for (kind, _) in &held.services {
        rows.push(spec(
            format!("accounts.{id}.service.{kind}"),
            section,
            kind,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
            toml::Value::String("on".to_owned()),
        ));
    }
    for (leaf, _) in &held.sync {
        let label = match leaf.as_str() {
            "files" => "Keep this account's files on this computer",
            _ => "Upload new photos from a folder on this computer to Google Photos",
        };
        rows.push(spec(
            format!("accounts.{id}.sync.{leaf}"),
            section,
            label,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
            toml::Value::String("off".to_owned()),
        ));
    }
    for (grant, what) in &held.grants {
        rows.push(spec(
            format!("accounts.{id}.grant.{grant}"),
            section,
            what,
            action("Revoke", ActionWeight::Plain),
            toml::Value::Boolean(false),
        ));
    }
    rows.push(spec(
        format!("accounts.{id}.reauth"),
        section,
        "Sign in again",
        action("Sign in again", ActionWeight::Plain),
        toml::Value::Boolean(false),
    ));
    rows.push(spec(
        format!("accounts.{id}.remove"),
        section,
        "Remove account",
        action("Remove account", ActionWeight::Destructive),
        toml::Value::Boolean(false),
    ));
    rows
}

impl LiveModule for Accountd {
    async fn permit(&self, _caller: &Caller, _access: Access) -> Verdict {
        match self.policy {
            Policy::Allow => Verdict::Allow,
            Policy::Refuse => Verdict::Refuse,
        }
    }

    async fn describe(&self) -> LiveSchema {
        let mut key: Vec<KeySpec> = self
            .accounts
            .lock()
            .expect("accounts")
            .iter()
            .flat_map(rows_of)
            .collect();
        let mut client = spec(
            "accounts.clients.microsoft".to_owned(),
            "Sign-in clients",
            "microsoft client id",
            KeyKind::Text,
            toml::Value::String(String::new()),
        );
        client.exposure = Exposure::Advanced;
        key.push(client);
        LiveSchema { version: 1, key }
    }

    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        let path = key.0.as_str();
        if path == "accounts.clients.microsoft" {
            return Ok(toml::Value::String(
                self.client.lock().expect("client").clone(),
            ));
        }
        let accounts = self.accounts.lock().expect("accounts");
        let on = accounts.iter().find_map(|held| {
            let services = held
                .services
                .iter()
                .map(|(k, on)| (format!("service.{k}"), on));
            let sync = held.sync.iter().map(|(k, on)| (format!("sync.{k}"), on));
            services.chain(sync).find_map(|(row, on)| {
                (path == format!("accounts.{}.{row}", held.id)).then_some(*on)
            })
        });
        match on {
            Some(on) => Ok(toml::Value::String(
                if on { "on" } else { "off" }.to_owned(),
            )),
            None => Ok(toml::Value::Boolean(false)),
        }
    }

    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        self.sets
            .lock()
            .expect("sets")
            .push((key.0.clone(), value.clone()));
        let path = key.0.as_str();
        if path == "accounts.clients.microsoft" {
            *self.client.lock().expect("client") = value.as_str().unwrap_or_default().to_owned();
            return Ok(());
        }
        let mut accounts = self.accounts.lock().expect("accounts");
        for held in accounts.iter_mut() {
            for (kind, on) in &mut held.services {
                if path == format!("accounts.{}.service.{kind}", held.id) {
                    *on = value.as_str() == Some("on");
                }
            }
            for (leaf, on) in &mut held.sync {
                if path == format!("accounts.{}.sync.{leaf}", held.id) {
                    *on = value.as_str() == Some("on");
                }
            }
            held.grants
                .retain(|(grant, _)| path != format!("accounts.{}.grant.{grant}", held.id));
        }
        accounts.retain(|held| path != format!("accounts.{}.remove", held.id));
        Ok(())
    }
}
