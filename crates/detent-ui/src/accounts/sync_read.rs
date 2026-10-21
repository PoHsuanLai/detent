//! What the sync service says of the datasets of the accounts shown: read again whenever the pane's
//! rows change and after "Sync now" was asked. A dataset the service does not answer for has no
//! entry, and nothing is said of it.

use crate::live::{BusSource, sync_status};
use detent_model::{AccountsState, SyncBook};
use dioxus::prelude::*;

/// The book of the datasets of `state`'s accounts, kept current. `again` is bumped to read once
/// more.
pub(super) fn use_sync_book(
    bus: BusSource,
    state: Signal<AccountsState>,
    again: Signal<u32>,
) -> Signal<SyncBook> {
    let book = use_signal(SyncBook::default);
    let _reading = use_resource(move || {
        let bus = bus.clone();
        let datasets = match state() {
            AccountsState::Ready(ready) => ready.pane.datasets(),
            AccountsState::Probing(_) | AccountsState::Unavailable(..) => Vec::new(),
        };
        let _asked = again();
        async move {
            let mut book = book;
            let mut answers = Vec::new();
            for dataset in datasets {
                if let Some(status) = sync_status(&bus, dataset.clone()).await {
                    answers.push((dataset, status));
                }
            }
            book.set(SyncBook::of(answers));
        }
    });
    book
}
