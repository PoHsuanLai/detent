//! Following one module for the pane: what it finds is told to the pane's state, and a row it
//! changes is heard as that module's.

use super::drive::drive;
use super::feeds::Feeds;
use crate::live::{Followed, follow};
use detent_model::{AccountsInput, AccountsState, ModuleId, accounts_row_is_valued};
use detent_system::accounts::AccountsLink;
use detent_system::bus::{BusSource, Daemon};
use dioxus::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::sync::Arc;

/// Follows `module` until the bus goes away. A module that is not there is reported at once and
/// looked for again when it appears; the others go on meanwhile.
pub(super) async fn follow_module(
    module: ModuleId,
    bus: BusSource,
    accounts: Arc<dyn AccountsLink>,
    state: Signal<AccountsState>,
    feeds: Feeds,
    toasts: ToastHub,
) {
    let mut client = feeds.of(module).client;
    follow(
        &bus,
        &Daemon::of(module),
        accounts_row_is_valued,
        |followed| {
            let run = |input| spawn(drive(state, feeds, accounts.clone(), toasts, input));
            match followed {
                Followed::Opened(opened, described) => {
                    client.set(Some(opened));
                    run(AccountsInput::Found(module, Ok(described)));
                }
                Followed::Gone(absence) => {
                    client.set(None);
                    run(AccountsInput::Found(module, Err(absence)));
                }
                Followed::Changed(path, value) => {
                    run(AccountsInput::heard(module, path, &value));
                }
            }
        },
    )
    .await;
}
