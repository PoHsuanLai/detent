//! The update service on the person's bus: `org.quire.Updates1`. It is only ever looked for, never
//! started: opening Settings must not wake a service the person's session did not start.

use super::link::{InstallRefusal, UpdateTicks, UpdateWork, Updater, UpdatesLink};
use super::wire::wire_of;
use crate::deadline::{Deadline, peer};
use crate::live::{BusSource, connect, name_owned};
use crate::ticks::{Ticker, Ticks};
use detent_model::{Absence, UPDATES_NOT_AVAILABLE, UnixSeconds, UpdateStatus, UpdateVersion};
use ds_peer::{NotAnswering, within};
use porter_dbus::{BusFailure, classify};
use std::collections::HashMap;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::Connection;
use zbus::export::futures_core::Stream;
use zbus::fdo::{DBusProxy, PropertiesProxy};
use zbus::names::InterfaceName;
use zbus::proxy::CacheProperties;
use zbus::zvariant::OwnedValue;

/// The name the service owns, and its object.
const BUS_NAME: &str = "org.quire.Updates1";
const PATH: &str = "/org/quire/Updates1";
const INTERFACE: &str = "org.quire.Updates1";

/// The error names the service answers a refusal with; each one's message is the plain sentence.
/// The first is the general failure (`Check`, and `Rollback`'s checks); the others are
/// `Install(version)`'s: the text named is not a version, or anything else that stopped it, such
/// as work in progress.
const SENTENCE_ERRORS: [&str; 3] = [
    "org.freedesktop.DBus.Error.Failed",
    "org.quire.Updates1.Error.NotAVersion",
    "org.quire.Updates1.Error.Refused",
];

/// The error name of `Install(version)` when what the service offers is not the version named,
/// or it offers nothing.
const VERSION_CHANGED: &str = "org.quire.Updates1.Error.VersionChanged";

/// What the person reads when a request failed and the service gave no sentence.
const COULD_NOT: &str = "The request could not be made. Try again in a moment.";

/// The service's side as this window calls it. The properties are read together, through
/// `GetAll` (`wire.rs`), so the proxy has only the requests.
#[zbus::proxy(
    interface = "org.quire.Updates1",
    default_service = "org.quire.Updates1",
    default_path = "/org/quire/Updates1"
)]
trait Updates {
    /// Looks for a newer version; returns at once.
    fn check(&self) -> zbus::Result<()>;
    /// Installs the newer version, which is `version`: the one the person was shown (the
    /// service's `Available`). Returns at once.
    fn install(&self, version: &str) -> zbus::Result<()>;
}

/// The one request the page makes of the service.
#[derive(Debug, Clone)]
enum Ask {
    Check,
    Install(UpdateVersion),
}

fn absence(error: zbus::Error) -> Absence {
    match classify(&error) {
        BusFailure::NoDaemon => Absence::NotRunning,
        BusFailure::Denied(_) => Absence::NotPermitted,
        _ => Absence::Failed(error.to_string()),
    }
}

/// The sentence for a request that failed: the service's own words when it refused, else the
/// general one.
fn sentence_of(error: &zbus::Error) -> String {
    match (classify(error), error) {
        (BusFailure::NoDaemon, _) => UPDATES_NOT_AVAILABLE.to_owned(),
        (_, zbus::Error::MethodError(name, Some(words), _))
            if SENTENCE_ERRORS.contains(&name.as_str()) && !words.is_empty() =>
        {
            words.clone()
        }
        (_, zbus::Error::FDO(inner)) => match inner.as_ref() {
            zbus::fdo::Error::Failed(words) if !words.is_empty() => words.clone(),
            _ => COULD_NOT.to_owned(),
        },
        _ => COULD_NOT.to_owned(),
    }
}

/// Why `Install(version)` failed: the version changed (by the service's error name), or the
/// sentence for the person.
fn refusal_of(error: &zbus::Error) -> InstallRefusal {
    match error {
        zbus::Error::MethodError(name, _, _) if name.as_str() == VERSION_CHANGED => {
            InstallRefusal::VersionChanged
        }
        _ => InstallRefusal::Told(sentence_of(error)),
    }
}

/// A connection, only when the service is already on the bus.
async fn running(bus: &BusSource) -> Result<Connection, Absence> {
    let connection = connect(bus).await?;
    if name_owned(&connection, BUS_NAME).await? {
        Ok(connection)
    } else {
        Err(Absence::NotRunning)
    }
}

/// The service's object, read live each time: a cached value would outlive the change.
async fn proxy(connection: &Connection) -> Result<UpdatesProxy<'_>, zbus::Error> {
    UpdatesProxy::builder(connection)
        .cache_properties(CacheProperties::No)
        .build()
        .await
}

/// Who did not answer, as the page says it.
fn silent(error: NotAnswering) -> Absence {
    Absence::Failed(error.to_string())
}

/// Every property of the service at once.
async fn get_all(connection: &Connection) -> Result<HashMap<String, OwnedValue>, Absence> {
    let properties = PropertiesProxy::builder(connection)
        .destination(BUS_NAME)
        .map_err(absence)?
        .path(PATH)
        .map_err(absence)?
        .build()
        .await
        .map_err(absence)?;
    let interface =
        InterfaceName::try_from(INTERFACE).map_err(|e| Absence::Failed(e.to_string()))?;
    properties
        .get_all(interface)
        .await
        .map_err(|error| absence(error.into()))
}

async fn read(bus: &BusSource) -> Result<UpdateStatus, Absence> {
    let connection = running(bus).await?;
    let all = within(Deadline::READ, peer::UPDATES, get_all(&connection))
        .await
        .map_err(silent)??;
    UpdateStatus::of_wire(wire_of(&all)?)
}

/// Tells `out` each time `stream` yields, until the stream ends or nobody listens.
async fn relay(mut stream: impl Stream<Item: Send> + Unpin + Send + 'static, out: Ticker) {
    while std::future::poll_fn(|cx| Pin::new(&mut stream).poll_next(cx))
        .await
        .is_some()
    {
        if out.tick().is_err() {
            return;
        }
    }
}

/// The ticks for the service's properties changing and for its name changing hands, subscribed
/// before anyone reads, so no change falls between the reading and the listening. The relays
/// end with the ticks.
async fn watch(bus: &BusSource) -> Result<UpdateTicks, Absence> {
    let connection = connect(bus).await?;
    let subscribed = async {
        let owners = DBusProxy::new(&connection)
            .await
            .map_err(absence)?
            .receive_name_owner_changed_with_args(&[(0, BUS_NAME)])
            .await
            .map_err(absence)?;
        let properties = PropertiesProxy::builder(&connection)
            .destination(BUS_NAME)
            .map_err(absence)?
            .path(PATH)
            .map_err(absence)?
            .build()
            .await
            .map_err(absence)?
            .receive_properties_changed()
            .await
            .map_err(absence)?;
        Ok::<_, Absence>((owners, properties))
    };
    let (owners, properties) = within(Deadline::READ, peer::DESKTOP, subscribed)
        .await
        .map_err(silent)??;
    let (ticker, ticks) = Ticks::channel();
    Ok(ticks
        .with_relay(tokio::spawn(relay(owners, ticker.clone())))
        .with_relay(tokio::spawn(relay(properties, ticker))))
}

async fn ask(bus: &BusSource, request: Ask) -> Result<(), InstallRefusal> {
    let told = InstallRefusal::Told;
    let connection = running(bus)
        .await
        .map_err(|_| told(UPDATES_NOT_AVAILABLE.to_owned()))?;
    let service = proxy(&connection)
        .await
        .map_err(|e| told(sentence_of(&e)))?;
    let call = async {
        match &request {
            Ask::Check => service.check().await,
            Ask::Install(version) => service.install(version.as_str()).await,
        }
    };
    within(Deadline::ACTION, peer::UPDATES, call)
        .await
        .map_err(|silent| told(silent.to_string()))?
        .map_err(|e| refusal_of(&e))
}

/// The words for the person when a request other than an install was refused.
fn words_of(refusal: InstallRefusal) -> String {
    match refusal {
        InstallRefusal::Told(words) => words,
        InstallRefusal::VersionChanged => COULD_NOT.to_owned(),
    }
}

/// The service on `bus`, and the system clock.
struct OnBus(BusSource);

impl UpdatesLink for OnBus {
    fn read(&self) -> UpdateWork<'_, Result<UpdateStatus, Absence>> {
        Box::pin(read(&self.0))
    }

    fn changes(&self) -> UpdateWork<'_, Result<UpdateTicks, Absence>> {
        Box::pin(watch(&self.0))
    }

    fn check(&self) -> UpdateWork<'_, Result<(), String>> {
        Box::pin(async move { ask(&self.0, Ask::Check).await.map_err(words_of) })
    }

    fn install(&self, version: &UpdateVersion) -> UpdateWork<'_, Result<(), InstallRefusal>> {
        Box::pin(ask(&self.0, Ask::Install(version.clone())))
    }

    fn now(&self) -> UnixSeconds {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| {
                i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
            });
        UnixSeconds(seconds)
    }
}

impl Updater {
    /// The update service on `bus`, looked for and never started.
    pub fn on(bus: BusSource) -> Updater {
        Updater::over(OnBus(bus))
    }
}
