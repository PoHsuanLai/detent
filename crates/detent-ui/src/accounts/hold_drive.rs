//! Running the held deletions' effects, and "Sync now", against the sync service: the one place the
//! pane calls it.

use crate::copy::account_sync::{HOLD_NOT_REMOVED, sync_now_line};
use crate::live::{BusSource, confirm_discard, hold_now, sync_now};
use detent_model::{DatasetName, HoldEffect, HoldInput, Holds, hold_step};
use dioxus::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::collections::VecDeque;

/// Applies `first` to the holds, then every input its effects lead to, until nothing is left to
/// do.
pub(super) async fn drive_holds(
    mut holds: Signal<Holds>,
    bus: BusSource,
    toasts: ToastHub,
    first: HoldInput,
) {
    let mut queue = VecDeque::from([first]);
    while let Some(input) = queue.pop_front() {
        let (next, effects) = hold_step(holds.peek().clone(), input);
        holds.set(next);
        for effect in effects {
            match effect {
                HoldEffect::Confirm(dataset) => {
                    queue.extend(confirm_discard(&bus, dataset).await);
                }
                HoldEffect::Refresh(dataset) => queue.push_back(hold_now(&bus, dataset).await),
                HoldEffect::CouldNotRemove => toasts.push(HOLD_NOT_REMOVED.to_owned(), None),
            }
        }
    }
}

/// Asks the sync service to sync `dataset` now and tells the person how it went.
pub(super) async fn drive_sync_now(bus: BusSource, toasts: ToastHub, dataset: DatasetName) {
    toasts.push(
        sync_now_line(sync_now(&bus, dataset).await).to_owned(),
        None,
    );
}
