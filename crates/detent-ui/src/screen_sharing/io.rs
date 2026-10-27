//! The calls on the portal: the list, and taking an app back. Both go to the owner of the
//! portal's name that was confirmed. The list starts the portal when nobody owns its name, since
//! the portal keeps the apps it lets share on disk; taking an app back never starts it.

use super::proxy::portal;
use super::wire::grants_of;
use crate::deadline::{Deadline, answered, peer};
use crate::live::{BusSource, absence_of_bus, pinned, pinned_starting};
use detent_callers::PORTAL_BUS;
use detent_model::{Absence, ProgramSubject, ScreenGrant};
use ds_peer::within;

/// What the person reads when an app could not be taken back.
const COULD_NOT_REMOVE: &str = "Screen sharing could not be changed. Try again in a moment.";

/// The apps that may share the screen without asking again. A portal nobody owns yet is asked to
/// start first, and its owner must hold the portal's role before it is called, as for any owner.
pub(super) async fn list(bus: &BusSource) -> Result<Vec<ScreenGrant>, Absence> {
    let running = pinned_starting(bus, PORTAL_BUS).await?;
    let proxy = portal(&running).await.map_err(|e| absence_of_bus(&e))?;
    let rows = within(Deadline::READ, peer::PORTAL, proxy.list_grants())
        .await
        .map_err(|_| Absence::Failed)?
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
