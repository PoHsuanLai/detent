//! The calls about looser settings and the confinement line. None starts the memory service: with it not running
//! nothing waits on anyone, and opening Settings must not start it.

use super::proxy::{MEMORY_NAME, control};
use super::{MemoryFailure, MemoryLink};
use crate::bus::{BusSource, Pinned, pinned};
use crate::deadline::{Deadline, answered, peer};
use almanac_core::WaitingSetting;
use detent_model::{Confinement, SettingPair, Trouble, Work};
use porter_dbus::{BusFailure, classify};

/// The bus name memory answers a refusal with when the caller may not do this.
const NOT_ALLOWED: &str = "org.quire.Memory1.Error.NotAllowed";

/// The bus name memory answers with when a pair is not one it takes.
const INVALID: &str = "org.quire.Memory1.Error.Invalid";

fn failure_of(error: &zbus::Error) -> MemoryFailure {
    if let zbus::Error::MethodError(name, _, _) = error {
        match name.as_str() {
            NOT_ALLOWED => return MemoryFailure::Trouble(Trouble::NotAllowed),
            INVALID => return MemoryFailure::Trouble(Trouble::Invalid),
            _ => {}
        }
    }
    match classify(error) {
        BusFailure::NoDaemon => MemoryFailure::Absent,
        BusFailure::Denied(_) => MemoryFailure::Trouble(Trouble::NotAllowed),
        _ => MemoryFailure::Trouble(Trouble::Failed),
    }
}

/// The connection and memory's owner, only when memory is already on the bus and its owner is
/// memory: a program that took the name is treated as memory not running, and is asked and told
/// nothing. The calls go to that owner.
async fn running(bus: &BusSource) -> Result<Pinned, MemoryFailure> {
    pinned(bus, MEMORY_NAME)
        .await
        .map_err(|_| MemoryFailure::Absent)
}

/// The looser values the file holds that wait for the person, in key order, read fresh.
async fn waiting(bus: &BusSource) -> Result<Vec<WaitingSetting>, MemoryFailure> {
    let memory = running(bus).await?;
    let proxy = control(&memory).await.map_err(|error| failure_of(&error))?;
    let rows = answered(Deadline::READ, peer::MEMORY, proxy.pending_settings())
        .await
        .ok_or(MemoryFailure::Absent)?
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
async fn confirm(bus: &BusSource, pairs: &[SettingPair]) -> Result<(), MemoryFailure> {
    let memory = running(bus).await?;
    let proxy = control(&memory).await.map_err(|error| failure_of(&error))?;
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
    .ok_or(MemoryFailure::Trouble(Trouble::Failed))?
    .map_err(|error| failure_of(&error))
}

/// How far memory confines itself, read fresh, only when memory is already running.
async fn confinement(bus: &BusSource) -> Result<Confinement, MemoryFailure> {
    let memory = running(bus).await?;
    let proxy = control(&memory).await.map_err(|error| failure_of(&error))?;
    let word = answered(Deadline::READ, peer::MEMORY, proxy.confinement())
        .await
        .ok_or(MemoryFailure::Absent)?
        .map_err(|error| failure_of(&error))?;
    Ok(Confinement::from_word(&word))
}

/// Memory on the bus it is built over.
#[derive(Debug)]
pub struct OnBus(pub BusSource);

impl MemoryLink for OnBus {
    fn waiting(&self) -> Work<'_, Result<Vec<WaitingSetting>, MemoryFailure>> {
        Box::pin(waiting(&self.0))
    }

    fn confirm(&self, pairs: Vec<SettingPair>) -> Work<'_, Result<(), MemoryFailure>> {
        Box::pin(async move { confirm(&self.0, &pairs).await })
    }

    fn confinement(&self) -> Work<'_, Result<Confinement, MemoryFailure>> {
        Box::pin(confinement(&self.0))
    }
}
