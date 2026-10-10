//! The sync service's held deletions: told as they start and end, asked for when a hold is
//! confirmed, never starting the service.

use super::bus::BusSource;
use super::daemon::Daemon;
use super::io::{connect, next_owner, owner_changes};
use detent_model::{DatasetName, HoldCounts, HoldInput};
use porter_dbus::names::SYNC_ERROR_NOTHING_HELD;
use porter_dbus::{BusError, SyncProxy};
use std::collections::HashMap;
use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;
use zbus::zvariant::OwnedValue;

/// The key of `Status`'s hold, and of the signal's counts.
const NEEDS_CONFIRMATION: &str = "needs_confirmation";
const DISCARD: &str = "discard";
const HELD: &str = "held";

/// The counts in a vardict `{discard: t, held: t, ..}`; none for an empty one (a hold that ended)
/// or one that lacks a count.
fn counts_of(details: &HashMap<String, OwnedValue>) -> Option<HoldCounts> {
    let count = |key: &str| details.get(key).and_then(|v| u64::try_from(v).ok());
    Some(HoldCounts {
        discard: count(DISCARD)?,
        held: count(HELD)?,
    })
}

/// A dataset's hold as `Status` says it: the nested `needs_confirmation` counts, or none.
fn held_in_status(status: &HashMap<String, OwnedValue>) -> Option<HoldCounts> {
    let nested = status.get(NEEDS_CONFIRMATION)?;
    let nested: HashMap<String, OwnedValue> = nested.try_clone().ok()?.try_into().ok()?;
    counts_of(&nested)
}

async fn syncd_is_running(connection: &Connection) -> bool {
    let (Ok(name), Ok(bus)) = (
        BusName::try_from(Daemon::syncd().name),
        DBusProxy::new(connection).await,
    ) else {
        return false;
    };
    bus.name_has_owner(name).await.unwrap_or(false)
}

/// Tells `tell` each hold the sync service starts or ends, until the bus goes away. It joins the
/// service with `Watch` once subscribed, so holds already on arrive at once, and again each time
/// the service's name changes hands (its list of listeners is in its memory). A service that is
/// not running is waited for, never started.
pub(crate) async fn follow_holds(bus: &BusSource, mut tell: impl FnMut(HoldInput)) {
    let Ok(connection) = connect(bus).await else {
        return;
    };
    let Ok(mut owners) = owner_changes(&connection, &Daemon::syncd()).await else {
        return;
    };
    loop {
        if syncd_is_running(&connection).await
            && watch(&connection, &mut owners, &mut tell).await.is_none()
        {
            return;
        }
        if next_owner(&mut owners).await.is_none() {
            return;
        }
    }
}

/// One stretch of the service being there: `Some` when its name changed hands, `None` when the
/// bus closed.
async fn watch(
    connection: &Connection,
    owners: &mut zbus::fdo::NameOwnerChangedStream,
    tell: &mut impl FnMut(HoldInput),
) -> Option<()> {
    let Ok(proxy) = SyncProxy::new(connection).await else {
        return next_owner(owners).await;
    };
    let Ok(mut signals) = proxy.receive_needs_confirmation().await else {
        return next_owner(owners).await;
    };
    // Holds already on come back as signals before this returns.
    let _ = proxy.watch().await;
    use zbus::export::futures_core::Stream;
    loop {
        tokio::select! {
            owner = next_owner(owners) => return owner,
            signal = std::future::poll_fn(|cx| std::pin::Pin::new(&mut signals).poll_next(cx)) => {
                let Some(signal) = signal else { return next_owner(owners).await };
                let Ok(args) = signal.args() else { continue };
                tell(HoldInput::Told {
                    dataset: DatasetName(args.dataset().to_string()),
                    counts: counts_of(args.held()),
                });
            }
        }
    }
}

/// Lets the discard through for `dataset`. Nothing to tell when the service took the word (the end
/// of the hold is told as a signal); `NothingHeld` when nothing was held any more; `Refused`
/// otherwise.
pub(crate) async fn confirm_discard(bus: &BusSource, dataset: DatasetName) -> Option<HoldInput> {
    let asked = async {
        let connection = connect(bus).await.ok()?;
        let proxy = SyncProxy::new(&connection).await.ok()?;
        Some(proxy.confirm_discard(&dataset.0).await)
    }
    .await;
    match asked {
        Some(Ok(())) => None,
        Some(Err(error)) if is_nothing_held(&error) => Some(HoldInput::NothingHeld(dataset)),
        Some(Err(_)) | None => Some(HoldInput::Refused(dataset)),
    }
}

fn is_nothing_held(error: &BusError) -> bool {
    matches!(error, BusError::MethodError(name, _, _) if name.as_str() == SYNC_ERROR_NOTHING_HELD)
}

/// `dataset`'s hold as `Status` says it now: told as `Told`, with no counts when nothing is held
/// or the dataset cannot be asked about.
pub(crate) async fn hold_now(bus: &BusSource, dataset: DatasetName) -> HoldInput {
    let counts = async {
        let connection = connect(bus).await.ok()?;
        let proxy = SyncProxy::new(&connection).await.ok()?;
        held_in_status(&proxy.status(&dataset.0).await.ok()?)
    }
    .await;
    HoldInput::Told { dataset, counts }
}
