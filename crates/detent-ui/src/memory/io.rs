//! The calls about looser settings and the confinement line. None starts the memory service: with it not running
//! nothing waits on anyone, and opening Settings must not start it.

use super::proxy::{MEMORY_NAME, control};
use crate::deadline::{Deadline, answered, peer};
use crate::live::{BusSource, Pinned, pinned};
use almanac_core::WaitingSetting;
use detent_model::{Confinement, SettingPair, Trouble};
use porter_dbus::{BusFailure, classify};

/// The bus name memory answers a refusal with when the caller may not do this.
const NOT_ALLOWED: &str = "org.quire.Memory1.Error.NotAllowed";

/// The bus name memory answers with when a pair is not one it takes.
const INVALID: &str = "org.quire.Memory1.Error.Invalid";

/// Why a call did not return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Failure {
    /// There is no bus, or memory is not running: there is nothing to say to the person.
    Absent,
    /// Memory (or the bus) refused or failed, in a way the person is told.
    Trouble(Trouble),
}

fn failure_of(error: &zbus::Error) -> Failure {
    if let zbus::Error::MethodError(name, _, _) = error {
        match name.as_str() {
            NOT_ALLOWED => return Failure::Trouble(Trouble::NotAllowed),
            INVALID => return Failure::Trouble(Trouble::Invalid),
            _ => {}
        }
    }
    match classify(error) {
        BusFailure::NoDaemon => Failure::Absent,
        BusFailure::Denied(_) => Failure::Trouble(Trouble::NotAllowed),
        BusFailure::Other(_) => Failure::Trouble(Trouble::Failed),
    }
}

/// The connection and memory's owner, only when memory is already on the bus and its owner is
/// memory: a program that took the name is treated as memory not running, and is asked and told
/// nothing. The calls go to that owner.
async fn running(bus: &BusSource) -> Result<Pinned, Failure> {
    pinned(bus, MEMORY_NAME).await.map_err(|_| Failure::Absent)
}

/// The looser values the file holds that wait for the person, in key order, read fresh.
pub(super) async fn waiting(bus: &BusSource) -> Result<Vec<WaitingSetting>, Failure> {
    let memory = running(bus).await?;
    let proxy = control(&memory)
        .await
        .map_err(|error| failure_of(&error))?;
    let rows = answered(Deadline::READ, peer::MEMORY, proxy.pending_settings())
        .await
        .ok_or(Failure::Absent)?
        .map_err(|error| failure_of(&error))?;
    Ok(rows
        .into_iter()
        .map(|(key, waiting, shipped)| WaitingSetting {
            key,
            waiting,
            shipped,
        })
        .collect())
}

/// Tells memory the person agreed to `pairs`; memory takes all of them or none.
pub(super) async fn confirm(bus: &BusSource, pairs: &[SettingPair]) -> Result<(), Failure> {
    let memory = running(bus).await?;
    let proxy = control(&memory)
        .await
        .map_err(|error| failure_of(&error))?;
    let values = pairs
        .iter()
        .map(|pair| (pair.key.clone(), pair.value.clone()))
        .collect();
    answered(
        Deadline::ACTION,
        peer::MEMORY,
        proxy.confirm_settings(values),
    )
    .await
    .ok_or(Failure::Trouble(Trouble::Failed))?
    .map_err(|error| failure_of(&error))
}

/// How far memory confines itself, read fresh, only when memory is already running.
pub(super) async fn confinement(bus: &BusSource) -> Result<Confinement, Failure> {
    let memory = running(bus).await?;
    let proxy = control(&memory)
        .await
        .map_err(|error| failure_of(&error))?;
    let word = answered(Deadline::READ, peer::MEMORY, proxy.confinement())
        .await
        .ok_or(Failure::Absent)?
        .map_err(|error| failure_of(&error))?;
    Ok(Confinement::from_word(&word))
}
