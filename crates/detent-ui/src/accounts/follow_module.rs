//! Following one module for the pane: what it finds is told to the pane's state, and a row it
//! changes is heard as that module's.

use super::drive::{drive, is_valued};
use super::feeds::Feeds;
use crate::live::{BusSource, Daemon, Followed, follow};
use detent_model::{AccountsInput, AccountsState, ModuleId};
use dioxus::prelude::*;
use ds::stack::toast_hub::ToastHub;

/// Follows `module` until the bus goes away. A module that is not there is reported at once and
/// looked for again when it appears; the others go on meanwhile.
pub(super) async fn follow_module(
    module: ModuleId,
    bus: BusSource,
    state: Signal<AccountsState>,
    feeds: Feeds,
    toasts: ToastHub,
) {
    let mut client = feeds.of(module).client;
    follow(&bus, &Daemon::of(module), is_valued, |followed| {
        let run = |input| spawn(drive(state, feeds, bus.clone(), toasts, input));
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
    })
    .await;
}
