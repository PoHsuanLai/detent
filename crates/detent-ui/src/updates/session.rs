//! The update service on the person's bus: `org.quire.Updates1`. It is only ever looked for, never
//! started: opening Settings must not wake a service the person's session did not start.

use super::link::{UpdateTicks, UpdateWork, Updater, UpdatesLink};
use crate::live::{BusSource, connect};
use detent_model::{Absence, UPDATES_NOT_AVAILABLE, UnixSeconds, UpdateStatus, UpdateWire};
use porter_dbus::{BusFailure, classify};
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use zbus::Connection;
use zbus::export::futures_core::Stream;
use zbus::fdo::{DBusProxy, PropertiesProxy};
use zbus::names::BusName;
use zbus::proxy::CacheProperties;

/// The name the service owns, and its object.
const BUS_NAME: &str = "org.quire.Updates1";
const PATH: &str = "/org/quire/Updates1";

/// The error name the service answers a refusal with; its message is the plain sentence.
const REFUSED: &str = "org.freedesktop.DBus.Error.Failed";

/// What the person reads when a request failed and the service gave no sentence.
const COULD_NOT: &str = "The request could not be made. Try again in a moment.";

/// The service's side as this window calls it.
#[zbus::proxy(
    interface = "org.quire.Updates1",
    default_service = "org.quire.Updates1",
    default_path = "/org/quire/Updates1"
)]
trait Updates {
    /// Looks for a newer version; returns at once.
    fn check(&self) -> zbus::Result<()>;
    /// Installs the newer version; returns at once.
    fn install(&self) -> zbus::Result<()>;
    /// Where the flow is.
    #[zbus(property)]
    fn state(&self) -> zbus::Result<String>;
    /// The installed version.
    #[zbus(property)]
    fn current(&self) -> zbus::Result<String>;
    /// The newer version, empty for none.
    #[zbus(property)]
    fn available(&self) -> zbus::Result<String>;
    /// Its notes.
    #[zbus(property)]
    fn notes(&self) -> zbus::Result<String>;
    /// Its download size in bytes.
    #[zbus(property)]
    fn size(&self) -> zbus::Result<u64>;
    /// Download progress in thousandths.
    #[zbus(property)]
    fn progress(&self) -> zbus::Result<u32>;
    /// When the last check finished, Unix seconds, 0 for never.
    #[zbus(property)]
    fn last_checked(&self) -> zbus::Result<i64>;
    /// What went wrong, in plain words, empty for nothing.
    #[zbus(property)]
    fn problem(&self) -> zbus::Result<String>;
}

/// The one request the page makes of the service.
#[derive(Debug, Clone, Copy)]
enum Ask {
    Check,
    Install,
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
            if name.as_str() == REFUSED && !words.is_empty() =>
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

/// A connection, only when the service is already on the bus.
async fn running(bus: &BusSource) -> Result<Connection, Absence> {
    let connection = connect(bus).await?;
    let name = BusName::try_from(BUS_NAME).map_err(|e| Absence::Failed(e.to_string()))?;
    let owned = DBusProxy::new(&connection)
        .await
        .map_err(absence)?
        .name_has_owner(name)
        .await
        .map_err(|e| Absence::Failed(e.to_string()))?;
    if owned {
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

async fn read(bus: &BusSource) -> Result<UpdateStatus, Absence> {
    let connection = running(bus).await?;
    let service = proxy(&connection).await.map_err(absence)?;
    let wire = UpdateWire {
        state: service.state().await.map_err(absence)?,
        current: service.current().await.map_err(absence)?,
        available: service.available().await.map_err(absence)?,
        notes: service.notes().await.map_err(absence)?,
        size: service.size().await.map_err(absence)?,
        progress: service.progress().await.map_err(absence)?,
        last_checked: service.last_checked().await.map_err(absence)?,
        problem: service.problem().await.map_err(absence)?,
    };
    UpdateStatus::of_wire(wire)
}

/// Tells `out` each time `stream` yields, until the stream ends or nobody listens.
async fn relay(
    mut stream: impl Stream<Item: Send> + Unpin + Send + 'static,
    out: UnboundedSender<()>,
) {
    while std::future::poll_fn(|cx| Pin::new(&mut stream).poll_next(cx))
        .await
        .is_some()
    {
        if out.send(()).is_err() {
            return;
        }
    }
}

/// The ticks for the service's properties changing and for its name changing hands, subscribed
/// before anyone reads, so no change falls between the reading and the listening.
async fn watch(bus: &BusSource) -> Result<UpdateTicks, Absence> {
    let connection = connect(bus).await?;
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
    let (out, ticks) = unbounded_channel();
    tokio::spawn(relay(owners, out.clone()));
    tokio::spawn(relay(properties, out));
    Ok(ticks)
}

async fn ask(bus: &BusSource, request: Ask) -> Result<(), String> {
    let connection = running(bus)
        .await
        .map_err(|_| UPDATES_NOT_AVAILABLE.to_owned())?;
    let service = proxy(&connection).await.map_err(|e| sentence_of(&e))?;
    let done = match request {
        Ask::Check => service.check().await,
        Ask::Install => service.install().await,
    };
    done.map_err(|e| sentence_of(&e))
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
        Box::pin(ask(&self.0, Ask::Check))
    }

    fn install(&self) -> UpdateWork<'_, Result<(), String>> {
        Box::pin(ask(&self.0, Ask::Install))
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
