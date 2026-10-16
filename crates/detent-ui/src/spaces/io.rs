//! The Spaces list and its removal, over accountd's registry and memory's controls.

use crate::live::{BusSource, connect};
use crate::memory::MemoryControlProxy;
use almanac_core::Removal;
use detent_model::{Absence, SpaceKey, SpaceRow};
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
    let spaces = DbusTransport::over(connection)
        .spaces()
        .await
        .map_err(absence_of)?;
    let records = spaces.list().await.map_err(absence_of)?;
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
/// Memory not being there means there is nothing to settle, and the registry is asked anyway.
pub(super) async fn remove(
    bus: &BusSource,
    space: &SpaceKey,
    removal: &Removal,
) -> Result<(), String> {
    let connection = connect(bus)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let wire = serde_json::to_string(removal).map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let memory = MemoryControlProxy::new(&connection)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    match memory.remove_space(&space.0, &wire).await {
        Ok(_) => {}
        Err(error) if matches!(classify(&error), BusFailure::NoDaemon) => {}
        Err(_) => return Err(COULD_NOT_REMOVE.to_owned()),
    }
    SpacesProxy::new(&connection)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?
        .remove(&space.0)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())
}
