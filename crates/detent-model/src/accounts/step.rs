//! The pane's one transition: `(state, input) -> (state, effects)`.

use super::model::{
    AccountKey, AccountsPane, AccountsReady, AccountsScreen, AccountsState, Ask, RemovalQuestion,
    toggle_word,
};
use crate::picker::{Absence, Described};
use ds_settings::schema::KeyPath;

/// What happened to the pane.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsInput {
    /// The module was read: the first time, or again after a change.
    Found(Result<Described, Absence>),
    /// The person opened an account.
    Open(AccountKey),
    /// The person went back to the list.
    Back,
    /// The person turned a service on or off.
    Switched {
        /// The service's row.
        path: KeyPath,
        /// What it is now.
        on: bool,
    },
    /// The person pressed a permission's action or Sign in again.
    Ran(KeyPath),
    /// The person asked to remove an account.
    AskRemove(KeyPath),
    /// The person asked to sign a coding agent's account out.
    AskSignOut(KeyPath),
    /// The person answered the question: no.
    Keep,
    /// The person answered the question: yes.
    Remove,
    /// The person finished a sign-in client id.
    ClientEdited {
        /// The row.
        path: KeyPath,
        /// What the field holds.
        text: String,
    },
    /// The module did what was asked at `path`.
    Done(KeyPath),
    /// The module refused what was asked at `path`.
    Failed {
        /// The row.
        path: KeyPath,
        /// What the module said.
        why: String,
    },
    /// The module says something changed.
    Changed,
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsEffect {
    /// Call `Set(path, value)` on the module.
    Set {
        /// The row.
        path: KeyPath,
        /// The value.
        value: toml::Value,
    },
    /// Run the action at `path`.
    Invoke(KeyPath),
    /// Read the module again.
    Reload,
    /// Tell the person something went wrong.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn accounts_step(
    state: AccountsState,
    input: AccountsInput,
) -> (AccountsState, Vec<AccountsEffect>) {
    match (state, input) {
        (AccountsState::Ready(ready), AccountsInput::Found(Ok(described))) => {
            (AccountsState::Ready(refreshed(ready, &described)), vec![])
        }
        (_, AccountsInput::Found(Ok(described))) => (
            AccountsState::Ready(AccountsReady::new(AccountsPane::of(&described))),
            vec![],
        ),
        (_, AccountsInput::Found(Err(absence))) => (AccountsState::Unavailable(absence), vec![]),
        (AccountsState::Ready(ready), input) => {
            let (ready, effects) = ready_step(ready, input);
            (AccountsState::Ready(ready), effects)
        }
        (state, _) => (state, vec![]),
    }
}

/// `ready` over a fresh reading: the screen stays unless its account is gone.
fn refreshed(ready: AccountsReady, described: &Described) -> AccountsReady {
    let pane = AccountsPane::of(described);
    let screen = match ready.screen {
        AccountsScreen::Detail(key) if pane.account(&key).is_some() => AccountsScreen::Detail(key),
        _ => AccountsScreen::List,
    };
    let confirming = ready
        .confirming
        .filter(|asked| pane.owner(&asked.path).is_some());
    AccountsReady {
        pane,
        screen,
        confirming,
        ..ready
    }
}

fn ready_step(
    mut ready: AccountsReady,
    input: AccountsInput,
) -> (AccountsReady, Vec<AccountsEffect>) {
    let mut effects = Vec::new();
    let idle = ready.pending.is_none();
    match input {
        AccountsInput::Open(key) if ready.pane.account(&key).is_some() => {
            ready.screen = AccountsScreen::Detail(key);
        }
        AccountsInput::Back => {
            ready.screen = AccountsScreen::List;
            ready.confirming = None;
        }
        AccountsInput::Switched { path, on } if idle => {
            // A service and a sync switch are both a path and a state: the same Set.
            let switch = ready
                .pane
                .accounts
                .iter_mut()
                .flat_map(|account| {
                    let services = account.services.iter_mut().map(|s| (&s.path, &mut s.on));
                    services.chain(account.sync.iter_mut().map(|s| (&s.path, &mut s.on)))
                })
                .find(|(row, held)| **row == path && **held != on);
            if let Some((_, held)) = switch {
                *held = on;
                effects.push(AccountsEffect::Set {
                    path: path.clone(),
                    value: toml::Value::String(toggle_word(on).to_owned()),
                });
                ready.pending = Some(path);
            }
        }
        AccountsInput::Ran(path) if idle && runs_without_asking(&ready.pane, &path) => {
            effects.push(AccountsEffect::Invoke(path.clone()));
            ready.pending = Some(path);
        }
        AccountsInput::AskRemove(path) if idle => {
            ready.confirming = ready.pane.owner(&path).and_then(|account| {
                account
                    .remove
                    .as_ref()
                    .filter(|(row, _)| *row == path)
                    .map(|_| RemovalQuestion {
                        ask: Ask::Remove,
                        path,
                        account: account.label.clone(),
                    })
            });
        }
        AccountsInput::AskSignOut(path) if idle => {
            ready.confirming = ready.pane.owner(&path).and_then(|account| {
                account
                    .sign_out
                    .as_ref()
                    .filter(|(row, _)| *row == path)
                    .map(|_| RemovalQuestion {
                        ask: Ask::SignOut,
                        path,
                        account: account.label.clone(),
                    })
            });
        }
        AccountsInput::Keep => ready.confirming = None,
        AccountsInput::Remove if idle => {
            if let Some(asked) = ready.confirming.take() {
                effects.push(AccountsEffect::Invoke(asked.path.clone()));
                ready.pending = Some(asked.path);
            }
        }
        AccountsInput::ClientEdited { path, text } if idle => {
            let text = text.trim().to_owned();
            let client = ready
                .pane
                .clients
                .iter_mut()
                .find(|client| client.path == path && client.value != text);
            if let Some(client) = client {
                client.value.clone_from(&text);
                effects.push(AccountsEffect::Set {
                    path: path.clone(),
                    value: toml::Value::String(text),
                });
                ready.pending = Some(path);
            }
        }
        AccountsInput::Done(path) => {
            if ready.pending.as_ref() == Some(&path) {
                ready.pending = None;
            }
            effects.push(AccountsEffect::Reload);
        }
        AccountsInput::Failed { path, why } => {
            if ready.pending.as_ref() == Some(&path) {
                ready.pending = None;
            }
            effects.push(AccountsEffect::Tell(why));
            effects.push(AccountsEffect::Reload);
        }
        AccountsInput::Changed => effects.push(AccountsEffect::Reload),
        _ => {}
    }
    (ready, effects)
}

/// Whether `path` is an action that may run on one press: a permission's or Sign in again.
/// Removing an account and signing an agent out are asked about first, so it is not one.
fn runs_without_asking(pane: &AccountsPane, path: &KeyPath) -> bool {
    pane.accounts.iter().any(|account| {
        account.permissions.iter().any(|p| &p.path == path)
            || account.reauth.as_ref().is_some_and(|(row, _)| row == path)
    })
}
