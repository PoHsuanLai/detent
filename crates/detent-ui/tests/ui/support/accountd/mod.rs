//! A fake accounts module: `org.quire.SettingsModule1` at accountd's, inferd's or syncd's path on a
//! private bus, with the rows the real modules describe (`accounts.<id>.service.<kind>`, `.grant.<grant>`, `.sync.<leaf>`, `.reauth`,
//! `.remove`, `accounts.clients.<issuer>`). A removed account and a revoked grant leave the rows.

mod module;
mod rows;

use super::bus::PrivateBus;
use super::syncd::Syncd;
use ds_settings::live::{
    Access, Caller, LiveError, LiveModule, LiveSchema, Served, Verdict, serve,
};
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};
use porter_dbus::names::{
    ACCOUNTS_BUS, ACCOUNTS_SETTINGS_PATH, INFERENCE_BUS, INFERENCE_SETTINGS_PATH, SYNC_BUS,
    SYNC_PATH,
};
use std::sync::{Arc, Mutex};

/// Which daemon's module the fake is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Module {
    Accountd,
    Inferd,
    Syncd,
}

impl Module {
    pub fn bus(self) -> &'static str {
        match self {
            Module::Accountd => ACCOUNTS_BUS,
            Module::Inferd => INFERENCE_BUS,
            Module::Syncd => SYNC_BUS,
        }
    }

    fn path(self) -> &'static str {
        match self {
            Module::Accountd => ACCOUNTS_SETTINGS_PATH,
            Module::Inferd => INFERENCE_SETTINGS_PATH,
            Module::Syncd => "/org/quire/Sync1/settings",
        }
    }
}

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
    /// The "Show in Files" switch with its state and help line: only for an account whose drive
    /// syncd can show.
    pub show_in_files: Option<(bool, String)>,
    /// Whether the account is a runtime on this computer: the `place` row says so.
    pub on_this_computer: bool,
    /// The words of the read-only `expires` row, for a sign-in that expires.
    pub expires: Option<String>,
    /// The slug of the read-only `sign_in` row, when the module publishes one.
    pub sign_in: Option<String>,
    /// The provider's id and name, for the read-only `provider` row.
    pub provider: Option<(String, String)>,
    /// The slug of the read-only `group` row, when the module publishes one.
    pub group: Option<String>,
    /// Each grant's read-only `since` day and `scope` word: `(grant, since, scope)`.
    pub grant_facts: Vec<(String, String, String)>,
    /// The names of the apps using the identity: the read-only `used_by` rows.
    pub used_by: Vec<String>,
    /// This month's cost and the cap, in cents: the read-only `spend.month` and `spend.cap` rows.
    pub spend: Option<(u64, u64)>,
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
            show_in_files: None,
            on_this_computer: false,
            expires: None,
            sign_in: None,
            provider: None,
            group: None,
            grant_facts: Vec::new(),
            used_by: Vec::new(),
            spend: None,
        }
    }

    /// The apps using the identity, by name.
    pub fn used_by(mut self, apps: &[&str]) -> Held {
        self.used_by = apps.iter().map(|a| (*a).to_owned()).collect();
        self
    }

    /// What the account has cost this month and the cap, in cents.
    pub fn spending(mut self, month: u64, cap: u64) -> Held {
        self.spend = Some((month, cap));
        self
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

    /// An account whose whole drive syncd can show in Files: the switch off, with `help` under it.
    pub fn showing_in_files(mut self, help: &str) -> Held {
        self.show_in_files = Some((false, help.to_owned()));
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

    /// The kind of account it is: the `provider` row says `id`, and `name` is what a person reads.
    pub fn of_provider(mut self, id: &str, name: &str) -> Held {
        self.provider = Some((id.to_owned(), name.to_owned()));
        self
    }

    /// The part of the page the account is listed under: the `group` row says `slug`.
    pub fn in_group(mut self, slug: &str) -> Held {
        self.group = Some(slug.to_owned());
        self
    }

    /// When grant `grant` was given and how long it holds: its `since` and `scope` rows.
    pub fn granted_on(mut self, grant: &str, since: &str, scope: &str) -> Held {
        self.grant_facts
            .push((grant.to_owned(), since.to_owned(), scope.to_owned()));
        self
    }

    pub fn granting(mut self, grant: &str, what: &str) -> Held {
        self.grants.push((grant.to_owned(), what.to_owned()));
        self
    }
}

/// The client secret the fake holds to begin with: a test looks for it on screen and finds none.
pub const SECRET: &str = "hunter2-hunter2";

/// The fake's state, shared with the test that watches it.
#[derive(Clone)]
pub struct Accountd {
    pub accounts: Arc<Mutex<Vec<Held>>>,
    /// The Microsoft client id the fake holds.
    pub client: Arc<Mutex<String>>,
    /// The Microsoft client secret the fake holds: `SECRET` to begin with.
    pub secret: Arc<Mutex<String>>,
    /// Every `Set` that arrived, in order.
    pub sets: Arc<Mutex<Vec<(String, toml::Value)>>>,
    /// How many calls (reads and sets) the module was asked to permit: what reached it at all.
    pub calls: Arc<Mutex<usize>>,
    policy: Policy,
    module: Module,
    /// Whether the module has syncd's `accounts.sync.add` row.
    continue_as: bool,
    /// The sync service's own object, served on this module's connection, when the module is
    /// syncd's.
    sync1: Option<Syncd>,
}

impl Accountd {
    pub fn new(policy: Policy, accounts: Vec<Held>) -> Accountd {
        Accountd::on(Module::Accountd, policy, accounts)
    }

    /// The module of `module`'s daemon.
    pub fn on(module: Module, policy: Policy, accounts: Vec<Held>) -> Accountd {
        Accountd {
            module,
            continue_as: false,
            sync1: None,
            accounts: Arc::new(Mutex::new(accounts)),
            client: Arc::default(),
            secret: Arc::new(Mutex::new(SECRET.to_owned())),
            sets: Arc::default(),
            calls: Arc::default(),
            policy,
        }
    }

    /// Gives the module syncd's `accounts.sync.add` row.
    pub fn continuing_as(mut self) -> Accountd {
        self.continue_as = true;
        self
    }

    /// Serves `Sync1` beside the module, so `Status` answers what `sync1` stands for.
    pub fn with_status(mut self, sync1: Syncd) -> Accountd {
        self.sync1 = Some(sync1);
        self
    }

    /// Serves the module on `bus`, owning its daemon's bus name; the connection must be kept alive.
    pub async fn serve_on(&self, bus: &PrivateBus) -> zbus::Connection {
        self.serve_announcing(bus).await.0
    }

    /// The same, with the means to announce a change the module did not get through `Set`.
    pub async fn serve_announcing(&self, bus: &PrivateBus) -> (zbus::Connection, Served) {
        let connection = bus.connect().await;
        let served = serve(&connection, self.module.path(), self.clone())
            .await
            .expect("serves");
        if let Some(sync1) = &self.sync1 {
            connection
                .object_server()
                .at(SYNC_PATH, sync1.clone())
                .await
                .expect("serves Sync1");
        }
        connection
            .request_name(self.module.bus())
            .await
            .expect("name");
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

    /// How many calls reached the module, of any kind.
    pub fn calls(&self) -> usize {
        *self.calls.lock().expect("calls")
    }
}
