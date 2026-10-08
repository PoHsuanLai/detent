//! Running the held deletions' effects against the sync service: the one place the pane calls it.

use crate::live::{BusSource, confirm_discard, hold_now};
use detent_model::{HoldEffect, HoldInput, Holds, hold_step};
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
                HoldEffect::Tell(text) => toasts.push(text, None),
            }
        }
    }
}
