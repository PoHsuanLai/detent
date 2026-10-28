//! The Spaces list and its removal, over accountd's registry and memory's controls. Every call
//! goes to the owner of the service's name that was confirmed, never to the name.

use super::SpacesLink;
use crate::bus::{BusSource, pinned_starting};
use crate::deadline::{Deadline, answered, peer};
use crate::memory_consent::control;
use almanac_core::Removal;
use detent_callers::{ACCOUNTS_BUS, MEMORY_BUS};
use detent_model::{Absence, SpaceKey, SpaceRow, Work, needs_memory};
use ds_peer::within;
use porter_client::SpacesError;
use porter_dbus::{BusFailure, SpacesProxy, classify};

/// What the person reads when a delete choice cannot be carried out because memory is not running.
const MEMORY_NOT_RUNNING: &str = "Memory isn\u{2019}t running right now, so this Space can\u{2019}t be deleted. Try again in a moment.";

/// What the person reads when a removal could not be done.
const COULD_NOT_REMOVE: &str = "The Space could not be removed. Try again in a moment.";

fn absence_of(error: SpacesError) -> Absence {
    match error {
        SpacesError::Unreachable => Absence::NotRunning,
        SpacesError::Denied(_) => Absence::NotPermitted,
        _ => Absence::Failed,
    }
}

/// The desktop-wide Spaces, in the order they were made.
async fn list(bus: &BusSource) -> Result<Vec<SpaceRow>, Absence> {
    let accountd = pinned_starting(bus, ACCOUNTS_BUS).await?;
    let asked = async {
        let spaces = accountd.accounts()?.spaces().await.map_err(absence_of)?;
        spaces.list().await.map_err(absence_of)
    };
    let records = within(Deadline::READ, peer::ACCOUNTS, asked)
        .await
        .map_err(|_| Absence::Failed)??;
    Ok(records
        .into_iter()
        .map(|record| SpaceRow {
            id: SpaceKey(record.id.as_str().to_owned()),
            name: record.name.as_str().to_owned(),
        })
        .collect())
}

/// Tells memory what becomes of the Space's memories and history, then asks the registry to
/// remove it. Memory first: asked the other way round, memory has already kept everything.
/// Memory not being there means there is nothing to settle when everything is kept, and the
/// registry is asked anyway; when the person chose to delete, nothing is done and they are told,
/// since only memory can delete what it holds. A program that took either name is told nothing.
async fn remove(bus: &BusSource, space: &SpaceKey, removal: &Removal) -> Result<(), String> {
    let wire = serde_json::to_string(removal).map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let accountd = pinned_starting(bus, ACCOUNTS_BUS)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    tell_memory(bus, space, &wire, removal).await?;
    let registry = SpacesProxy::builder(&accountd.connection)
        .destination(accountd.destination())
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?
        .build()
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    answered(Deadline::ACTION, peer::ACCOUNTS, registry.remove(&space.0))
        .await
        .ok_or_else(|| COULD_NOT_REMOVE.to_owned())?
        .map_err(|_| COULD_NOT_REMOVE.to_owned())
}

/// Tells memory the person's choice for the Space; with no memory running, what the choice does
/// without it.
async fn tell_memory(
    bus: &BusSource,
    space: &SpaceKey,
    wire: &str,
    removal: &Removal,
) -> Result<(), String> {
    let memory = match pinned_starting(bus, MEMORY_BUS).await {
        Ok(memory) => memory,
        Err(Absence::NotRunning) => return without_memory(removal),
        Err(_) => return Err(COULD_NOT_REMOVE.to_owned()),
    };
    let proxy = control(&memory)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let told = answered(
        Deadline::ACTION,
        peer::MEMORY,
        proxy.remove_space(&space.0, wire),
    )
    .await;
    match told {
        Some(Ok(_)) => Ok(()),
        Some(Err(error)) if matches!(classify(&error), BusFailure::NoDaemon) => {
            without_memory(removal)
        }
        Some(Err(_)) | None => Err(COULD_NOT_REMOVE.to_owned()),
    }
}

/// What a removal does when memory is not there: carries on when it keeps everything, else stops.
fn without_memory(removal: &Removal) -> Result<(), String> {
    if needs_memory(removal) {
        Err(MEMORY_NOT_RUNNING.to_owned())
    } else {
        Ok(())
    }
}

/// accountd's registry and memory's controls on the bus they are built over.
#[derive(Debug)]
pub struct OnBus(pub BusSource);

impl SpacesLink for OnBus {
    fn list(&self) -> Work<'_, Result<Vec<SpaceRow>, Absence>> {
        Box::pin(list(&self.0))
    }

    fn remove(&self, space: SpaceKey, removal: Removal) -> Work<'_, Result<(), String>> {
        Box::pin(async move { remove(&self.0, &space, &removal).await })
    }
}
