//! What the sync service says of a dataset (`Sync1.Status`): when it last synced, what is left, and
//! how full the account's storage is. Asked of the owner that was confirmed, never starting the
//! service; a dataset it does not know, or a service that is not there, gives nothing.

use super::holds::{sync_of, syncd_running};
use crate::bus::BusSource;
use crate::deadline::{Deadline, answered, peer};
use detent_model::{DatasetName, StorageQuota, SyncPause, SyncStatus};
use porter_dbus::STATUS_KEY_QUOTA;
use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

/// `Status`' keys other than the quota's.
const ANCHOR_AGE: &str = "anchor_age";
const PENDING: &str = "pending";
const CONFLICTS: &str = "conflicts";
const PAUSED: &str = "paused";
/// The quota's two counts.
const USED: &str = "used";
const TOTAL: &str = "total";

fn number(details: &HashMap<String, OwnedValue>, key: &str) -> Option<u64> {
    details.get(key).and_then(|v| u64::try_from(v).ok())
}

/// The quota in `{used: t, total: t}`; `total` is missing when the provider reports no limit.
fn quota_of(details: &HashMap<String, OwnedValue>) -> Option<StorageQuota> {
    let nested = details.get(STATUS_KEY_QUOTA)?;
    let nested: HashMap<String, OwnedValue> = nested.try_clone().ok()?.try_into().ok()?;
    Some(StorageQuota {
        used: number(&nested, USED)?,
        total: number(&nested, TOTAL),
    })
}

/// A dataset's status as `Status` words it. `pending` is always there in the service's answer, so
/// an answer without it is not one.
fn status_of(details: &HashMap<String, OwnedValue>) -> Option<SyncStatus> {
    let paused = details.get(PAUSED).and_then(|v| bool::try_from(v).ok());
    Some(SyncStatus {
        anchor_age: details.get(ANCHOR_AGE).and_then(|v| i64::try_from(v).ok()),
        pending: number(details, PENDING)?,
        conflicts: number(details, CONFLICTS).unwrap_or_default(),
        pause: match paused {
            Some(true) => SyncPause::Paused,
            Some(false) | None => SyncPause::Running,
        },
        quota: quota_of(details),
    })
}

/// `dataset`'s status now.
pub async fn sync_status(bus: &BusSource, dataset: DatasetName) -> Option<SyncStatus> {
    answered(Deadline::READ, peer::SYNC, async {
        let syncd = syncd_running(bus).await?;
        let proxy = sync_of(&syncd).await?;
        status_of(&proxy.status(&dataset.0).await.ok()?)
    })
    .await
    .flatten()
}
