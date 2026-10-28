//! The sync service's held deletions: told as they start and end, asked for when a hold is
//! confirmed, never starting the service.

use crate::bus::{BusSource, Daemon, Pinned, connect, next_owner, owner_changes, pinned};
use crate::deadline::{Deadline, answered, peer};
use detent_model::{DatasetName, HoldCounts, HoldInput, SyncNowEnd};
use porter_dbus::BusError;
use porter_dbus::sync::SyncProxy;
use porter_dbus::sync::{SYNC_ERROR_NOTHING_HELD, SYNC_ERROR_PAUSED};
use std::collections::HashMap;
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

/// The sync service, only when it is on the bus and its owner is the sync service: a program that
/// took the name is not listened to and is told nothing. The calls go to that owner.
pub async fn syncd_running(bus: &BusSource) -> Option<Pinned> {
    pinned(bus, Daemon::syncd().name).await.ok()
}

/// The sync service's side, called by the owner that was confirmed.
pub async fn sync_of(syncd: &Pinned) -> Option<SyncProxy<'_>> {
    SyncProxy::builder(&syncd.connection)
        .destination(syncd.destination())
        .ok()?
        .build()
        .await
        .ok()
}

/// Tells `tell` each hold the sync service starts or ends, until the bus goes away. It joins the
/// service with `Watch` once subscribed, so holds already on arrive at once, and again each time
/// the service's name changes hands (its list of listeners is in its memory). A service that is
/// not running is waited for, never started.
pub async fn follow_holds(bus: &BusSource, mut tell: impl FnMut(HoldInput)) {
    let Ok(connection) = connect(bus).await else {
        return;
    };
    let Ok(mut owners) = owner_changes(&connection, &Daemon::syncd()).await else {
        return;
    };
    loop {
        if let Some(syncd) = syncd_running(bus).await
            && watch(&syncd, &mut owners, &mut tell).await.is_none()
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
    syncd: &Pinned,
    owners: &mut zbus::fdo::NameOwnerChangedStream,
    tell: &mut impl FnMut(HoldInput),
) -> Option<()> {
    let Some(proxy) = sync_of(syncd).await else {
        return next_owner(owners).await;
    };
    let Ok(mut signals) = proxy.receive_needs_confirmation().await else {
        return next_owner(owners).await;
    };
    // Holds already on come back as signals before this returns.
    let _ = answered(Deadline::READ, peer::SYNC, proxy.watch()).await;
    use zbus::export::futures_core::Stream;
    loop {
        tokio::select! {
            owner = next_owner(owners) => return owner,
            signal = std::future::poll_fn(|cx| std::pin::Pin::new(&mut signals).poll_next(cx)) => {
                let Some(signal) = signal else { return next_owner(owners).await };
                let Ok(args) = signal.args() else { continue };
                // A dataset name over its bound is not one the service makes: the signal is dropped.
                let Some(dataset) = DatasetName::bounded(args.dataset()) else { continue };
                tell(HoldInput::Told {
                    dataset,
                    counts: counts_of(args.held()),
                });
            }
        }
    }
}

/// Lets the discard through for `dataset`. Nothing to tell when the service took the word (the end
/// of the hold is told as a signal); `NothingHeld` when nothing was held any more; `Refused`
/// otherwise.
pub async fn confirm_discard(bus: &BusSource, dataset: DatasetName) -> Option<HoldInput> {
    let asked = answered(Deadline::ACTION, peer::SYNC, async {
        let syncd = syncd_running(bus).await?;
        let proxy = sync_of(&syncd).await?;
        Some(proxy.confirm_discard(&dataset.0).await)
    })
    .await
    .flatten();
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
pub async fn hold_now(bus: &BusSource, dataset: DatasetName) -> HoldInput {
    let counts = answered(Deadline::READ, peer::SYNC, async {
        let syncd = syncd_running(bus).await?;
        let proxy = sync_of(&syncd).await?;
        held_in_status(&proxy.status(&dataset.0).await.ok()?)
    })
    .await
    .flatten();
    HoldInput::Told { dataset, counts }
}

fn is_paused(error: &BusError) -> bool {
    matches!(error, BusError::MethodError(name, _, _) if name.as_str() == SYNC_ERROR_PAUSED)
}

/// Has the sync service sync `dataset` at once. It answers when it took the request, not when the
/// cycle ends.
pub async fn sync_now(bus: &BusSource, dataset: DatasetName) -> SyncNowEnd {
    let asked = answered(Deadline::ACTION, peer::SYNC, async {
        let syncd = syncd_running(bus).await?;
        let proxy = sync_of(&syncd).await?;
        Some(proxy.sync_now(&dataset.0).await)
    })
    .await
    .flatten();
    match asked {
        Some(Ok(())) => SyncNowEnd::Started,
        Some(Err(error)) if is_paused(&error) => SyncNowEnd::Paused,
        Some(Err(_)) | None => SyncNowEnd::Failed,
    }
}
