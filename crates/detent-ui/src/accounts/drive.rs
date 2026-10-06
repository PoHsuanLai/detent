//! Running the pane's effects against accountd: the one place the pane calls the bus.

use crate::live::{BusSource, Daemon, reread};
use detent_model::{
    Absence, AccountsEffect, AccountsInput, AccountsPane as Pane, AccountsState, accounts_step,
};
use dioxus::prelude::*;
use ds::stack::toast_hub::ToastHub;
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyKind;
use std::collections::VecDeque;

/// The rows whose value is read with the rows: the services (on or off) and the client ids.
pub(super) fn is_valued(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Toggle { .. } | KeyKind::Text)
}

/// The accounts accountd lists now, read once: what the intents provider answers from. Never
/// starts accountd.
pub async fn read_accounts(bus: &BusSource) -> Result<Pane, Absence> {
    let described = crate::live::read_once(bus, &Daemon::accountd(), is_valued).await?;
    Ok(Pane::of(&described))
}

/// Applies `first` to the pane, then every input its effects lead to, until nothing is left to
/// do. The calls to accountd wait here, so a caller spawns this and returns.
pub(super) async fn drive(
    mut state: Signal<AccountsState>,
    client: Signal<Option<LiveClient>>,
    toasts: ToastHub,
    first: AccountsInput,
) {
    let mut queue = VecDeque::from([first]);
    while let Some(input) = queue.pop_front() {
        let (next, effects) = accounts_step(state.peek().clone(), input);
        state.set(next);
        let opened = client.peek().clone();
        for effect in effects {
            match (effect, &opened) {
                (AccountsEffect::Tell(text), _) => toasts.push(text, None),
                (AccountsEffect::Reload, Some(opened)) => {
                    queue.push_back(AccountsInput::Found(reread(opened, is_valued).await));
                }
                (AccountsEffect::Set { path, value }, Some(opened)) => {
                    queue.push_back(answer(path.clone(), opened.set(&path, &value).await));
                }
                (AccountsEffect::Invoke(path), Some(opened)) => {
                    queue.push_back(answer(path.clone(), opened.invoke(&path).await));
                }
                (_, None) => {}
            }
        }
    }
}

fn answer(
    path: ds_settings::schema::KeyPath,
    result: Result<(), ds_settings::live::LiveError>,
) -> AccountsInput {
    match result {
        Ok(()) => AccountsInput::Done(path),
        Err(error) => AccountsInput::Failed {
            path,
            why: error.to_string(),
        },
    }
}
