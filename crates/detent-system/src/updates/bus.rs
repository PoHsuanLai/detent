//! The update service on the person's bus: `org.quire.Updates1`. It is only ever looked for, never
//! started: opening Settings must not wake a service the person's session did not start.

use super::wire::wire_of;
use super::{InstallRefusal, UpdateTicks, UpdatesLink};
use crate::bus::{BusSource, Pinned, connect, pinned, relay};
use crate::deadline::{Deadline, peer};
use crate::ticks::Ticks;
use detent_model::{Absence, UnixSeconds, UpdateStatus, UpdateVersion, Work};
use ds_peer::{NotAnswering, within};
use porter_dbus::{BusFailure, classify};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
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
        _ => Absence::Failed,
    }
}

/// What the person reads when no service answers.
fn not_available() -> String {
    "Software updates are not available on this computer.".to_owned()
}

/// The sentence for a request that failed: the service's own words when it refused, else the
/// general one.
fn sentence_of(error: &zbus::Error) -> String {
    match (classify(error), error) {
        (BusFailure::NoDaemon, _) => not_available(),
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

/// The service's object, read live each time: a cached value would outlive the change. It is
/// called by the owner that was confirmed to be the update service.
async fn proxy(running: &Pinned) -> Result<UpdatesProxy<'_>, zbus::Error> {
    UpdatesProxy::builder(&running.connection)
        .destination(running.destination())?
        .cache_properties(CacheProperties::No)
        .build()
        .await
}

/// Who did not answer, as the page says it.
fn silent(_error: NotAnswering) -> Absence {
    Absence::Failed
}

/// Every property of the service at once.
async fn get_all(running: &Pinned) -> Result<HashMap<String, OwnedValue>, Absence> {
    let properties = PropertiesProxy::builder(&running.connection)
        .destination(running.destination())
        .map_err(absence)?
        .path(PATH)
        .map_err(absence)?
        .build()
        .await
        .map_err(absence)?;
    let interface = InterfaceName::try_from(INTERFACE).map_err(|_| Absence::Failed)?;
    properties
        .get_all(interface)
        .await
        .map_err(|error| absence(error.into()))
}

async fn read(bus: &BusSource) -> Result<UpdateStatus, Absence> {
    let running = pinned(bus, BUS_NAME).await?;
    let all = within(Deadline::READ, peer::UPDATES, get_all(&running))
        .await
        .map_err(silent)??;
    UpdateStatus::of_wire(wire_of(&all)?)
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
    let running = pinned(bus, BUS_NAME)
        .await
        .map_err(|_| told(not_available()))?;
    let service = proxy(&running).await.map_err(|e| told(sentence_of(&e)))?;
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

/// The service on the bus it is built over, and the system clock.
#[derive(Debug)]
pub struct OnBus(pub BusSource);

impl UpdatesLink for OnBus {
    fn read(&self) -> Work<'_, Result<UpdateStatus, Absence>> {
        Box::pin(read(&self.0))
    }

    fn changes(&self) -> Work<'_, Result<UpdateTicks, Absence>> {
        Box::pin(watch(&self.0))
    }

    fn check(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async move { ask(&self.0, Ask::Check).await.map_err(words_of) })
    }

    fn install(&self, version: &UpdateVersion) -> Work<'_, Result<(), InstallRefusal>> {
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

#[cfg(test)]
mod tests {
    /// The name this page calls is the one the owner check knows the role of.
    #[test]
    fn the_service_name_is_the_one_the_owner_check_knows() {
        assert_eq!(super::BUS_NAME, detent_callers::UPDATES_BUS);
    }
}
