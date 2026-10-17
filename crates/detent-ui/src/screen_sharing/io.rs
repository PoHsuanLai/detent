//! The calls on the portal: the list, and taking an app back. Both go to the owner of the
//! portal's name that was confirmed, and neither starts the portal.

use super::proxy::portal;
use super::wire::grants_of;
use crate::deadline::{Deadline, answered, peer};
use crate::live::{BusSource, absence_of_bus, pinned};
use detent_callers::PORTAL_BUS;
use detent_model::{Absence, ProgramSubject, ScreenGrant};
use ds_peer::within;

/// What the person reads when an app could not be taken back.
const COULD_NOT_REMOVE: &str = "Screen sharing could not be changed. Try again in a moment.";

/// The apps that may share the screen without asking again.
pub(super) async fn list(bus: &BusSource) -> Result<Vec<ScreenGrant>, Absence> {
    let running = pinned(bus, PORTAL_BUS).await?;
    let proxy = portal(&running).await.map_err(|e| absence_of_bus(&e))?;
    let rows = within(Deadline::READ, peer::PORTAL, proxy.list_grants())
        .await
        .map_err(|silent| Absence::Failed(silent.to_string()))?
        .map_err(|e| absence_of_bus(&e))?;
    Ok(grants_of(&rows))
}

/// Takes `app` back: the portal asks the person the next time it wants the screen. The portal's
/// count of what went is of no use here; none is as good as one, the app is gone from the list.
pub(super) async fn revoke(bus: &BusSource, app: &ProgramSubject) -> Result<(), String> {
    let running = pinned(bus, PORTAL_BUS)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    let proxy = portal(&running)
        .await
        .map_err(|_| COULD_NOT_REMOVE.to_owned())?;
    answered(Deadline::ACTION, peer::PORTAL, proxy.revoke(&app.0))
        .await
        .ok_or_else(|| COULD_NOT_REMOVE.to_owned())?
        .map(|_| ())
        .map_err(|_| COULD_NOT_REMOVE.to_owned())
}
