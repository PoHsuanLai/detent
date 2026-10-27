//! Running the pane's effects against the three modules: the one place the pane calls the bus.
//! A row's effect goes to the module the row came from.

use super::feeds::Feeds;
use super::reload::{reload, reload_later};
use crate::copy::account_notices::account_notice;
use detent_model::{
    AccountsEffect, AccountsInput, AccountsState, ModuleId, ModuleRow, accounts_step,
};
use detent_system::accounts::AccountsLink;
use detent_system::bus::plain;
use dioxus::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::collections::VecDeque;
use std::sync::Arc;

/// Applies `first` to the pane, then every input its effects lead to, until nothing is left to
/// do. The calls to the modules wait here, so a caller spawns this and returns. Reads of a
/// module go through its own `reads`, one at a time.
pub(super) async fn drive(
    mut state: Signal<AccountsState>,
    feeds: Feeds,
    accounts: Arc<dyn AccountsLink>,
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
                AccountsEffect::Tell(notice) => toasts.push(account_notice(&notice), None),
                AccountsEffect::OpenAddSheet => {
                    let end = accounts.add_account().await;
                    queue.push_back(AccountsInput::AddEnded(ModuleId::Accountd, end));
                }
                AccountsEffect::OpenAiKeySheet => {
                    let end = accounts.add_ai_key().await;
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
