//! Running the pane's effects against the three modules: the one place the pane calls the bus.
//! A row's effect goes to the module the row came from.

use super::add_io::{add_account, add_ai_key};
use super::feeds::Feeds;
use super::reload::{reload, reload_later};
use crate::deadline::{Deadline, peer};
use crate::live::{BusSource, Daemon, plain};
use detent_model::{
    Absence, AccountsEffect, AccountsInput, AccountsPane as Pane, AccountsState, ModuleId,
    ModuleRow, Sources, accounts_step,
};
use dioxus::prelude::*;
use ds::stack::toast_hub::ToastHub;
use ds_peer::within;
use ds_settings::schema::KeyKind;
use std::collections::VecDeque;

/// The rows whose value is read with the rows: the services (on or off) and the client ids.
pub(crate) fn is_valued(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Toggle { .. } | KeyKind::Text)
}

/// The accounts the modules list now, read once: what the intents provider answers from. Never
/// starts a daemon. A module that is not there leaves its part out; the answer is an absence
/// only when none is there.
///
/// The whole read is under one action deadline besides each call's own: three modules of a
/// row's worth of calls each must not hold an agent's request for as long as every call could.
pub async fn read_accounts(bus: &BusSource) -> Result<Pane, Absence> {
    within(Deadline::ACTION, peer::ACCOUNTS, read_modules(bus))
        .await
        .map_err(|silent| Absence::Failed(silent.to_string()))?
}

async fn read_modules(bus: &BusSource) -> Result<Pane, Absence> {
    let mut sources = Sources::default();
    for module in ModuleId::ALL {
        let found = crate::live::read_once(bus, &Daemon::of(module), is_valued).await;
        sources.hear(module, found);
    }
    match sources.pane() {
        Some(pane) => Ok(pane),
        None => Err(sources.absence().unwrap_or(Absence::NotRunning)),
    }
}

/// Applies `first` to the pane, then every input its effects lead to, until nothing is left to
/// do. The calls to the modules wait here, so a caller spawns this and returns. Reads of a
/// module go through its own `reads`, one at a time.
pub(super) async fn drive(
    mut state: Signal<AccountsState>,
    feeds: Feeds,
    bus: BusSource,
    toasts: ToastHub,
    first: AccountsInput,
) {
    let mut queue = VecDeque::from([first]);
    while let Some(input) = queue.pop_front() {
        let (next, effects) = accounts_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            let opened = |module: ModuleId| feeds.of(module).client.peek().clone();
            match effect {
                AccountsEffect::Tell(text) => toasts.push(text, None),
                AccountsEffect::OpenAddSheet => {
                    let end = add_account(&bus).await;
                    queue.push_back(AccountsInput::AddEnded(ModuleId::Accountd, end));
                }
                AccountsEffect::OpenAiKeySheet => {
                    let end = add_ai_key(&bus).await;
                    queue.push_back(AccountsInput::AddEnded(ModuleId::Inferd, end));
                }
                AccountsEffect::Show(key) => queue.push_back(AccountsInput::Open(key)),
                AccountsEffect::Reload(module) => {
                    if let Some(client) = opened(module) {
                        let found = reload(feeds.of(module).reads, &client).await;
                        queue.extend(found.map(|found| AccountsInput::Reread(module, found)));
                    }
                }
                AccountsEffect::Retry(module) => {
                    if let Some(client) = opened(module) {
                        let found = reload_later(feeds.of(module).reads, &client).await;
                        queue.extend(found.map(|found| AccountsInput::Reread(module, found)));
                    }
                }
                AccountsEffect::Set { row, value } => {
                    if let Some(client) = opened(row.module) {
                        let result = client.set(&row.path, &value).await;
                        queue.push_back(answer(row, result));
                    }
                }
                AccountsEffect::SetSecret { row, secret } => {
                    if let Some(client) = opened(row.module) {
                        let result = client
                            .set(&row.path, &toml::Value::String(secret.into_text()))
                            .await;
                        queue.push_back(answer(row, result));
                    }
                }
                AccountsEffect::Invoke(row) => {
                    if let Some(client) = opened(row.module) {
                        let result = client.invoke(&row.path).await;
                        queue.push_back(answer(row, result));
                    }
                }
            }
        }
    }
}

fn answer(row: ModuleRow, result: Result<(), ds_settings::live::LiveError>) -> AccountsInput {
    match result {
        Ok(()) => AccountsInput::Done(row),
        Err(error) => AccountsInput::Failed {
            row,
            why: plain(&error),
        },
    }
}
