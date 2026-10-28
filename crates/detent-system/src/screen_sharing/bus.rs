//! The calls on the portal: the list, and taking an app back. Both go to the owner of the
//! portal's name that was confirmed. The list starts the portal when nobody owns its name, since
//! the portal keeps the apps it lets share on disk; taking an app back never starts it.

use super::ScreenSharingLink;
use super::proxy::portal;
use crate::bus::{BusSource, ToAbsence, pinned, pinned_starting};
use crate::deadline::{Deadline, answered, peer};
use detent_callers::PORTAL_BUS;
use detent_model::{Absence, ProgramSubject, ScreenGrant, Work, screen_grants_of};
use ds_peer::within;

/// What the person reads when an app could not be taken back.
const COULD_NOT_REMOVE: &str = "Screen sharing could not be changed. Try again in a moment.";

/// The apps that may share the screen without asking again. A portal nobody owns yet is asked to
/// start first, and its owner must hold the portal's role before it is called, as for any owner.
async fn list(bus: &BusSource) -> Result<Vec<ScreenGrant>, Absence> {
    let running = pinned_starting(bus, PORTAL_BUS).await?;
    let proxy = portal(&running).await.map_err(|e| e.absence())?;
    let rows = within(Deadline::READ, peer::PORTAL, proxy.list_grants())
        .await
        .map_err(|_| Absence::Failed)?
        .map_err(|e| e.absence())?;
    Ok(screen_grants_of(&rows))
}

/// Takes `app` back: the portal asks the person the next time it wants the screen. The portal's
/// count of what went is of no use here; none is as good as one, the app is gone from the list.
async fn revoke(bus: &BusSource, app: &ProgramSubject) -> Result<(), String> {
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

/// The portal on the bus it is built over.
#[derive(Debug)]
pub struct OnBus(pub BusSource);

impl ScreenSharingLink for OnBus {
    fn list(&self) -> Work<'_, Result<Vec<ScreenGrant>, Absence>> {
        Box::pin(list(&self.0))
    }

    fn revoke(&self, app: ProgramSubject) -> Work<'_, Result<(), String>> {
        Box::pin(async move { revoke(&self.0, &app).await })
    }
}
