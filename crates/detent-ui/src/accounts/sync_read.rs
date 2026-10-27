//! What the sync service says of the datasets of the accounts shown: read again whenever the
//! pane's rows change. A dataset the service does not answer for has no entry, and nothing is
//! said of it. The reading is `detent-client`'s.

use detent_client::Datasets;
use detent_model::AccountsState;
use dioxus::prelude::*;

/// Keeps the datasets' book current for the accounts of `state`.
pub(super) fn use_sync_reading(datasets: Datasets, state: Signal<AccountsState>) {
    let _reading = use_resource(move || {
        let datasets = datasets.clone();
        let names = match state() {
            AccountsState::Ready(ready) => ready.pane.datasets(),
            AccountsState::Probing(_) | AccountsState::Unavailable(..) => Vec::new(),
        };
        async move { datasets.read(names).await }
    });
}
