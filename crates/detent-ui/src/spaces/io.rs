//! The Spaces list and its removal, over accountd's registry and memory's controls.

use crate::deadline::{Deadline, answered, peer};
use crate::live::{BusSource, connect, not_squatted};
use crate::memory::MemoryControlProxy;
use almanac_core::Removal;
use detent_callers::{ACCOUNTS_BUS, MEMORY_BUS};
use detent_model::{Absence, MEMORY_NOT_RUNNING, SpaceKey, SpaceRow, WithoutMemory};
use ds_peer::within;
use porter_client::{DbusTransport, SpacesError};
use porter_dbus::{BusFailure, SpacesProxy, classify};

/// What the person reads when a removal could not be done.
const COULD_NOT_REMOVE: &str = "The Space could not be removed. Try again in a moment.";

fn absence_of(error: SpacesError) -> Absence {
    match error {
        SpacesError::Unreachable => Absence::NotRunning,
        SpacesError::Denied(_) => Absence::NotPermitted,
        other => Absence::Failed(other.to_string()),
    }
}

/// The desktop-wide Spaces, in the order they were made.
pub(super) async fn list(bus: &BusSource) -> Result<Vec<SpaceRow>, Absence> {
    let connection = connect(bus).await?;
    not_squatted(bus, &connection, ACCOUNTS_BUS).await?;
    let asked = async {
        let spaces = DbusTransport::over(connection)
            .spaces()
            .await
            .map_err(absence_of)?;
        spaces.list().await.map_err(absence_of)
    };
    let records = within(Deadline::READ, peer::ACCOUNTS, asked)
        .await
        .map_err(|silent| Absence::Failed(silent.to_string()))??;
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
/// since only memory can delete what it holds.
pub(super) async fn remove(
    bus: &BusSource,
    space: &SpaceKey,
    removal: &Removal,
) -> Result<(), String> {
    let connection = connect(bus)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let wire = serde_json::to_string(removal).map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    // A program that took memory's or the registry's name is told nothing; a name nobody owns is
    // still asked by name, which may start the service.
    not_squatted(bus, &connection, MEMORY_BUS)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    not_squatted(bus, &connection, ACCOUNTS_BUS)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let memory = MemoryControlProxy::new(&connection)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let told = answered(
        Deadline::ACTION,
        peer::MEMORY,
        memory.remove_space(&space.0, &wire),
    )
    .await;
    match told {
        Some(Ok(_)) => {}
        Some(Err(error)) if matches!(classify(&error), BusFailure::NoDaemon) => {
            if WithoutMemory::of(removal) == WithoutMemory::Refuse {
                return Err(MEMORY_NOT_RUNNING.to_owned());
            }
        }
        Some(Err(_)) | None => return Err(COULD_NOT_REMOVE.to_owned()),
    }
    let registry = SpacesProxy::new(&connection)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    answered(Deadline::ACTION, peer::ACCOUNTS, registry.remove(&space.0))
        .await
        .ok_or_else(|| COULD_NOT_REMOVE.to_owned())?
        .map_err(|_| COULD_NOT_REMOVE.to_owned())
}
