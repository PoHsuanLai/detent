//! Settings served on a private bus, called the way intentd calls an app: from the connection
//! that owns `org.quire.Intents1`, through `IntentProvider1`.
#![allow(dead_code)]

mod bus;

use detent_callers::{BusPeers, RoleName, introduce_service, private_peers, roles};
use detent_files::ConfigHome;
use detent_intents::{
    AccountsRead, AccountsSource, SettingsProvider, Window, WindowGone, serve_with,
};
use detent_model::{Absence, AccountTarget, AccountsPane, Catalogue, PageSlug, discover};
use detent_testkit::{Scratch, seed_schemas_settable};
use docket_core::{
    AppRefusal, Args, CallId, EntityRef, Hit, Invocation, Origin, Outcome, ParamName, Preview,
    SuggestAsk, UndoFault, UndoToken, Value,
};
use docket_dbus::callers::{INTENTD, Identity, intentd_only};
use docket_dbus::{Details, INTENTS_BUS, IntentProviderProxy};
use prov::{Actor, EntityId, Label, Labelled, Source, SpaceId};
use std::sync::{Arc, Mutex};

pub use bus::PrivateBus;

pub const TMP: &str = env!("CARGO_TARGET_TMPDIR");
const APP: &str = "org.quire.Settings";

/// What the window was told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Told {
    Page(String),
    Files,
    Account(AccountTarget),
    Raised(Option<String>),
}

/// A window that writes down what it is told.
#[derive(Default)]
pub struct WrittenDown(Mutex<Vec<Told>>);

impl WrittenDown {
    pub fn told(&self) -> Vec<Told> {
        self.0.lock().expect("lock").clone()
    }
}

impl Window for WrittenDown {
    fn open_page(&self, page: PageSlug) -> Result<(), WindowGone> {
        self.0.lock().expect("lock").push(Told::Page(page.0));
        Ok(())
    }
    fn files_changed(&self) -> Result<(), WindowGone> {
        self.0.lock().expect("lock").push(Told::Files);
        Ok(())
    }
    fn open_account(&self, target: AccountTarget) -> Result<(), WindowGone> {
        self.0.lock().expect("lock").push(Told::Account(target));
        Ok(())
    }
    fn raise(&self, token: Option<String>) -> Result<(), WindowGone> {
        self.0.lock().expect("lock").push(Told::Raised(token));
        Ok(())
    }
}

/// accountd's answer, whatever it is asked: a list of accounts, or why there is none.
pub struct Listed(pub Result<AccountsPane, Absence>);

impl AccountsSource for Listed {
    fn read(&self) -> AccountsRead<'_> {
        Box::pin(std::future::ready(self.0.clone()))
    }
}

/// The Settings provider serving, and intentd's stand-in calling it.
pub struct World {
    pub window: Arc<WrittenDown>,
    scratch: Scratch,
    proxy: IntentProviderProxy<'static>,
    _bus: PrivateBus,
    _app: zbus::Connection,
    _intentd: zbus::Connection,
    /// What the activation object judges its callers with: nobody is anyone until a test says so.
    peers: Arc<BusPeers>,
}

pub fn entity(kind: &str, key: &str) -> EntityId {
    EntityId {
        app: prov::AppName::parse(APP).expect("app"),
        kind: prov::EntityKind::parse(kind).expect("kind"),
        key: prov::EntityKey::parse(key).expect("key"),
    }
}

fn space() -> SpaceId {
    SpaceId::parse("work").expect("space")
}

fn from_the_terminal<T>(value: T) -> Labelled<T> {
    Labelled::new(
        value,
        Label::untrusted(Source::Cli, porter_core::DataClass::AppOwn, space()),
    )
}

/// A call as `quire-do` makes it.
pub fn invocation(action: &str, args: &[(&str, Value)]) -> Invocation {
    let args: Args = args
        .iter()
        .map(|(name, value)| {
            (
                ParamName::parse(name).expect("param"),
                from_the_terminal(value.clone()),
            )
        })
        .collect();
    Invocation {
        call: CallId(1),
        action: prov::ActionName::parse(action).expect("action"),
        target: docket_core::TargetValue::Nothing,
        args,
        actor: Actor::Cli,
        origin: Origin::Cli,
        space: space(),
    }
}

pub fn key(path: &str) -> Value {
    Value::Entity(entity("settings.key", path))
}

pub fn text(words: &str) -> Value {
    Value::Text(words.to_owned())
}

impl World {
    /// Settings serving the fixture schemas, with `settable` marked agent-settable, over a
    /// scratch config home.
    pub async fn start(test: &str, settable: &[&str]) -> World {
        World::start_listing(test, settable, Listed(Ok(AccountsPane::default()))).await
    }

    /// The same, with accountd answering `accounts`.
    pub async fn start_listing(test: &str, settable: &[&str], accounts: Listed) -> World {
        let scratch = Scratch::under(TMP, test);
        let schemas = seed_schemas_settable(&scratch.join("data"), settable);
        let catalogue = Arc::new(Catalogue::build(&discover(&[schemas]).programs));
        let window = Arc::new(WrittenDown::default());
        let provider = SettingsProvider::new(
            catalogue,
            ConfigHome(scratch.join("config")),
            window.clone(),
            Arc::new(accounts),
        )
        .expect("a provider");

        let bus = PrivateBus::start(scratch.path());
        let app = bus.connect().await;
        // Every connection of a test is one process, so intentd's stand-in is introduced as the
        // router the provider answers rather than found by its unit.
        let who = Arc::new(Identity::on_fixture(
            app.clone(),
            intentd_only(),
            scratch.join("proc"),
        ));
        let peers = Arc::new(private_peers(&app));
        serve_with(&app, provider, who.clone(), peers.clone())
            .await
            .expect("settings serves");
        let intentd = bus.connect().await;
        who.introduce_app(&intentd, INTENTD.app, INTENTD.role);
        intentd
            .request_name(INTENTS_BUS)
            .await
            .expect("intentd's name");
        let proxy = IntentProviderProxy::builder(&intentd)
            .destination(APP)
            .expect("destination")
            .build()
            .await
            .expect("proxy");
        World {
            window,
            scratch,
            proxy,
            _bus: bus,
            _app: app,
            _intentd: intentd,
            peers,
        }
    }

    /// A connection to the bus, as the launcher (sill) has: the shell, which calls the app by name.
    pub async fn launcher(&self) -> zbus::Connection {
        self.caller(roles::SHELL).await
    }

    /// A connection to the bus that the desktop's caller table would know as `role`.
    pub async fn caller(&self, role: RoleName) -> zbus::Connection {
        let connection = self.stranger().await;
        introduce_service(&self.peers, &unique(&connection), &role);
        connection
    }

    /// A connection to the bus that is in no row of the caller table: any program of the person's.
    pub async fn stranger(&self) -> zbus::Connection {
        self._bus.connect().await
    }

    /// The unique name of the connection that serves Settings.
    pub fn owner(&self) -> String {
        unique(&self._app)
    }

    pub fn config_home(&self) -> ConfigHome {
        ConfigHome(self.scratch.join("config"))
    }

    pub fn read(&self, relative: &str) -> Option<String> {
        std::fs::read_to_string(self.config_home().0.join(relative)).ok()
    }

    pub fn write(&self, relative: &str, text: &str) {
        let path = self.config_home().0.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    }

    pub async fn perform(
        &self,
        action: &str,
        args: &[(&str, Value)],
    ) -> Result<Outcome, AppRefusal> {
        let json = serde_json::to_string(&invocation(action, args)).expect("json");
        let answer = self
            .proxy
            .perform(&json, &Details::new())
            .await
            .expect("the bus call");
        serde_json::from_str(&answer).expect("an Outcome or an AppRefusal")
    }

    pub async fn dry_run(
        &self,
        action: &str,
        args: &[(&str, Value)],
    ) -> Result<Preview, AppRefusal> {
        let json = serde_json::to_string(&invocation(action, args)).expect("json");
        let answer = self
            .proxy
            .dry_run(&json, &Details::new())
            .await
            .expect("the bus call");
        serde_json::from_str(&answer).expect("a Preview or an AppRefusal")
    }

    pub async fn undo(&self, token: &UndoToken) -> Result<(), UndoFault> {
        let token = serde_json::to_string(token).expect("json");
        let actor = serde_json::to_string(&Actor::Cli).expect("json");
        let answer = self.proxy.undo(&token, &actor).await.expect("the bus call");
        serde_json::from_str(&answer).expect("() or an UndoFault")
    }

    pub async fn suggest(&self, action: &str, param: &str, typed: &str) -> Vec<EntityRef> {
        let ask = SuggestAsk {
            action: docket_core::ActionRef {
                app: prov::AppName::parse(APP).expect("app"),
                name: prov::ActionName::parse(action).expect("action"),
            },
            param: ParamName::parse(param).expect("param"),
            typed: typed.to_owned(),
        };
        let json = serde_json::to_string(&ask).expect("json");
        let answer = self.proxy.suggest(&json).await.expect("the bus call");
        serde_json::from_str(&answer).expect("entities")
    }

    pub async fn search(&self, words: &str) -> Vec<Hit> {
        let answer = self.proxy.search(words, 1).await.expect("the bus call");
        serde_json::from_str(&answer).expect("hits")
    }

    pub async fn preview(&self, id: &EntityId) -> Preview {
        let json = serde_json::to_string(id).expect("json");
        let answer = self.proxy.preview(&json).await.expect("the bus call");
        serde_json::from_str(&answer).expect("a Preview")
    }
}

/// The unique bus name of `connection`.
pub fn unique(connection: &zbus::Connection) -> String {
    connection
        .unique_name()
        .expect("a connection to a bus has a unique name")
        .to_string()
}

/// Peers on `connection` that know the connection named `owner` as `role`: what a program on
/// that connection judges the owner of a name with.
pub fn peers_knowing(connection: &zbus::Connection, owner: &str, role: &RoleName) -> BusPeers {
    let peers = private_peers(connection);
    introduce_service(&peers, owner, role);
    peers
}
