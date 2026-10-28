//! The real modules link over a module served on a private bus: the module is found, its rows are
//! read with their values, and a row set through the handle comes back as a change the module sent.

use crate::private_bus::PrivateBus;
use detent_model::Absence;
use detent_system::bus::{BusSource, Daemon, Vouch};
use detent_system::modules::{Followed, ModulesLink, OnBus};
use ds_settings::live::{
    Access, AnyOwner, Caller, LiveError, LiveModule, LiveSchema, Verdict, serve,
};
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section,
};
use std::sync::Mutex;
use tokio::sync::mpsc::unbounded_channel;

/// The bus name and path the module is served at.
const NAME: &str = "org.quire.TestModules1";
const PATH: &str = "/org/quire/TestModules1/settings";

/// The one row the module has.
pub(crate) const ROW: &str = "words.one";

/// A module of one text row.
pub(crate) struct Words {
    value: Mutex<String>,
    policy: Verdict,
}

impl Words {
    pub(crate) fn holding(word: &str, policy: Verdict) -> Words {
        Words {
            value: Mutex::new(word.to_owned()),
            policy,
        }
    }
}

impl LiveModule for Words {
    async fn permit(&self, _caller: &Caller, _access: Access) -> Verdict {
        self.policy
    }

    async fn describe(&self) -> LiveSchema {
        LiveSchema {
            version: 1,
            key: vec![KeySpec {
                ends: None,
                control: None,
                path: KeyPath(ROW.to_owned()),
                kind: KeyKind::Text,
                default: toml::Value::String(String::new()),
                label: Label("One".to_owned()),
                help: Help(String::new()),
                page: Page::Accounts,
                section: Section("Words".to_owned()),
                exposure: Exposure::Basic,
                labels: Default::default(),
                unavailable: Default::default(),
                groups: Default::default(),
                places: Default::default(),
                note: None,
                agent: AgentSetting::HandsOff,
            }],
        }
    }

    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        match key.0.as_str() {
            ROW => Ok(toml::Value::String(
                self.value.lock().expect("value").clone(),
            )),
            other => Err(LiveError::UnknownKey(other.to_owned())),
        }
    }

    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        match (key.0.as_str(), value.as_str()) {
            (ROW, Some(word)) => {
                *self.value.lock().expect("value") = word.to_owned();
                Ok(())
            }
            _ => Err(LiveError::BadValue("a word for the row".to_owned())),
        }
    }
}

/// The rows whose value is read with the rows.
pub(crate) fn is_text(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Text)
}

/// The module's daemon, on the bus the test made.
pub(crate) fn daemon() -> Daemon {
    Daemon {
        name: NAME,
        path: PATH.to_owned(),
    }
}

/// What a follower is told, and the module's handle once it is opened.
async fn opened(
    heard: &mut tokio::sync::mpsc::UnboundedReceiver<Followed>,
) -> (detent_system::modules::Module, String) {
    match heard.recv().await {
        Some(Followed::Opened(module, described)) => {
            let word = described.values.get(&KeyPath(ROW.to_owned())).cloned();
            (module, word.expect("the row's value is read with the rows"))
        }
        other => panic!("expected the module to open, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_module_on_the_bus_is_opened_and_its_changes_are_heard() {
    let bus = PrivateBus::start("modules-follow");
    let connection = bus.connect().await;
    serve(&connection, PATH, Words::holding("first", Verdict::Allow))
        .await
        .expect("serves");
    connection.request_name(NAME).await.expect("name");
    let link = OnBus(BusSource::Address(bus.address.clone(), Vouch::of(AnyOwner)));
    let daemon = daemon();
    let (told, mut heard) = unbounded_channel();
    let following = link.follow(&daemon, is_text, told);
    let checking = async {
        let (module, word) = opened(&mut heard).await;
        assert_eq!(word, "first");
        let row = KeyPath(ROW.to_owned());
        module
            .set(&row, &toml::Value::String("second".to_owned()))
            .await
            .expect("the module takes the word");
        match heard.recv().await {
            Some(Followed::Changed(path, value)) => {
                assert_eq!(path, row);
                assert_eq!(value, toml::Value::String("second".to_owned()));
            }
            other => panic!("expected the change, got {other:?}"),
        }
    };
    tokio::select! {
        () = following => panic!("the follower ended while the bus was there"),
        () = checking => {}
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_module_that_refuses_the_settings_app_is_gone_as_not_permitted() {
    let bus = PrivateBus::start("modules-refused");
    let connection = bus.connect().await;
    serve(&connection, PATH, Words::holding("first", Verdict::Refuse))
        .await
        .expect("serves");
    connection.request_name(NAME).await.expect("name");
    let link = OnBus(BusSource::Address(bus.address.clone(), Vouch::of(AnyOwner)));
    let daemon = daemon();
    let (told, mut heard) = unbounded_channel();
    let following = link.follow(&daemon, is_text, told);
    let checking = async {
        match heard.recv().await {
            Some(Followed::Gone(why)) => assert_eq!(why, Absence::NotPermitted),
            other => panic!("expected the module to be refused, got {other:?}"),
        }
    };
    tokio::select! {
        () = following => panic!("the follower ended while the bus was there"),
        () = checking => {}
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn looking_for_a_module_that_is_not_there_says_so_and_never_starts_it() {
    let bus = PrivateBus::start("modules-absent");
    let link = OnBus(BusSource::Address(bus.address.clone(), Vouch::of(AnyOwner)));
    let daemon = daemon();
    let (told, mut heard) = unbounded_channel();
    let following = link.follow(&daemon, is_text, told);
    let checking = async {
        match heard.recv().await {
            Some(Followed::Gone(why)) => assert_eq!(why, Absence::NotRunning),
            other => panic!("expected the module to be missing, got {other:?}"),
        }
        let connection = bus.connect().await;
        let names = zbus::fdo::DBusProxy::new(&connection)
            .await
            .expect("proxy")
            .list_names()
            .await
            .expect("names");
        assert!(
            !names.iter().any(|name| name.as_str() == NAME),
            "nothing owns the module's name: the link only looked"
        );
    };
    tokio::select! {
        () = following => panic!("the follower ended while the bus was there"),
        () = checking => {}
    }
}
