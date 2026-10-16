//! The pane's one transition: `(state, input) -> (state, effects)`.

use super::add_account::{AddEnd, Adding};
use super::model::{
    AccountKey, AccountsPane, AccountsReady, AccountsScreen, AccountsState, Ask, LocalFiles,
    RemovalQuestion, toggle_word,
};
use super::module::{ModuleId, ModuleRow, Sources};
use super::reauth_end::ReauthEnd;
use super::sign_in_client::{ClientChange, ClientText, Secret, SignInClient};
use super::sign_out::SignOutEnd;
use crate::picker::{Absence, Described};

/// What happened to the pane.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsInput {
    /// A module was found: the first time it was read, or when it came back; or it is not there.
    Found(ModuleId, Result<Described, Absence>),
    /// A module was read again after a change; a failed read keeps what is shown.
    Reread(ModuleId, Result<Described, Absence>),
    /// The person opened an account.
    Open(AccountKey),
    /// The person chose an account in the list.
    Select(AccountKey),
    /// The person went back to the list.
    Back,
    /// The person turned a service on or off.
    Switched {
        /// The service's row.
        row: ModuleRow,
        /// What it is now.
        on: bool,
    },
    /// The person pressed a permission's action or Sign in again.
    Ran(ModuleRow),
    /// The person asked to remove an account.
    AskRemove(ModuleRow),
    /// The person asked to sign a assistant's account out.
    AskSignOut(ModuleRow),
    /// The person answered the question: no.
    Keep,
    /// The person answered the question: yes.
    Remove,
    /// The person finished a sign-in client id.
    ClientEdited {
        /// The row.
        row: ModuleRow,
        /// What the field holds.
        text: ClientText,
    },
    /// The module did what was asked at `path`.
    Done(ModuleRow),
    /// The module refused what was asked at `path`.
    Failed {
        /// The row.
        row: ModuleRow,
        /// What the module said.
        why: String,
    },
    /// The person pressed Add Account….
    AddAccount,
    /// The person pressed Add AI Key….
    AddAiKey,
    /// The add sheet ended; it was the one of `module`.
    AddEnded(ModuleId, AddEnd),
    /// The module says something changed.
    Changed(ModuleId),
    /// The module says how the sign-in the person started at `path` ended.
    SignedIn {
        /// The account's reauth row.
        row: ModuleRow,
        /// How it ended.
        end: ReauthEnd,
    },
    /// The module says how an assistant's sign-out at `path` ended.
    SignedOut {
        /// The account's sign-out row.
        row: ModuleRow,
        /// How it ended.
        end: SignOutEnd,
    },
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsEffect {
    /// Call `Set(path, value)` on the module.
    Set {
        /// The row.
        row: ModuleRow,
        /// The value.
        value: toml::Value,
    },
    /// Call `Set(path, text)` on the module with a sign-in secret, which is never read back.
    SetSecret {
        /// The row.
        row: ModuleRow,
        /// The secret.
        secret: Secret,
    },
    /// Run the action at the row.
    Invoke(ModuleRow),
    /// Open the add sheet for an identity and tell how it ended as `AddEnded`.
    OpenAddSheet,
    /// Open the add sheet for an AI key and tell how it ended as `AddEnded`.
    OpenAiKeySheet,
    /// Read the module again.
    Reload(ModuleId),
    /// Read the module again after a short wait: the last read failed.
    Retry(ModuleId),
    /// Open the account with this key once the rows are read again.
    Show(AccountKey),
    /// Tell the person something went wrong.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn accounts_step(
    state: AccountsState,
    input: AccountsInput,
) -> (AccountsState, Vec<AccountsEffect>) {
    match input {
        AccountsInput::Found(module, found) => (hear(state, module, found), vec![]),
        AccountsInput::Reread(module, Ok(described)) => {
            (hear(state, module, Ok(described)), vec![])
        }
        // A read that fails while the module's rows are shown keeps them; the module moved
        // under it.
        AccountsInput::Reread(module, Err(_)) => {
            let shown = sources_of(&state).has_read(module);
            let effects = if shown {
                vec![AccountsEffect::Retry(module)]
            } else {
                vec![]
            };
            (state, effects)
        }
        AccountsInput::Changed(module) if matches!(state, AccountsState::Unavailable(..)) => {
            (state, vec![AccountsEffect::Reload(module)])
        }
        input => match state {
            AccountsState::Ready(ready) => {
                let (ready, effects) = ready_step(*ready, input);
                (AccountsState::Ready(Box::new(ready)), effects)
            }
            state => (state, vec![]),
        },
    }
}

fn sources_of(state: &AccountsState) -> &Sources {
    match state {
        AccountsState::Probing(sources) | AccountsState::Unavailable(_, sources) => sources,
        AccountsState::Ready(ready) => &ready.sources,
    }
}

/// `state` once `module` has said what it found. Rows of a module that is gone are left out;
/// the pane is unavailable only when every module is.
fn hear(
    state: AccountsState,
    module: ModuleId,
    found: Result<Described, Absence>,
) -> AccountsState {
    let (mut sources, ready) = match state {
        AccountsState::Probing(sources) | AccountsState::Unavailable(_, sources) => (sources, None),
        AccountsState::Ready(ready) => (ready.sources.clone(), Some(*ready)),
    };
    sources.hear(module, found);
    match (sources.pane(), ready) {
        (Some(pane), Some(ready)) => {
            AccountsState::Ready(Box::new(refreshed(ready, pane, sources)))
        }
        (Some(pane), None) => AccountsState::Ready(Box::new(AccountsReady::new(pane, sources))),
        (None, _) => match sources.absence() {
            Some(absence) => AccountsState::Unavailable(absence, sources),
            None => AccountsState::Probing(sources),
        },
    }
}

/// `ready` over a fresh reading: the screen stays unless its account is gone.
fn refreshed(ready: AccountsReady, pane: AccountsPane, sources: Sources) -> AccountsReady {
    let screen = match ready.screen {
        AccountsScreen::Detail(key) if pane.account(&key).is_some() => AccountsScreen::Detail(key),
        _ => AccountsScreen::List,
    };
    let confirming = ready
        .confirming
        .filter(|asked| pane.owner(&asked.row).is_some());
    let selected = ready.selected.filter(|key| pane.account(key).is_some());
    AccountsReady {
        pane,
        screen,
        selected,
        confirming,
        sources,
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
        AccountsInput::Select(key) if ready.pane.account(&key).is_some() => {
            ready.selected = Some(key);
        }
        AccountsInput::Back => {
            ready.screen = AccountsScreen::List;
            ready.confirming = None;
        }
        AccountsInput::Switched { row, on } if idle => {
            // A service and a sync switch are both a path and a state: the same Set.
            let switch = ready
                .pane
                .accounts
                .iter_mut()
                .filter(|account| account.module == row.module)
                .flat_map(|account| {
                    let services = account.services.iter_mut().map(|s| (&s.path, &mut s.on));
                    services.chain(account.sync.iter_mut().map(|s| (&s.path, &mut s.on)))
                })
                .find(|(path, held)| **path == row.path && **held != on);
            if let Some((_, held)) = switch {
                *held = on;
                effects.push(AccountsEffect::Set {
                    row: row.clone(),
                    value: toml::Value::String(toggle_word(on).to_owned()),
                });
                ready.pending = Some(row);
            }
        }
        AccountsInput::Ran(row) if idle && runs_without_asking(&ready.pane, &row) => {
            effects.push(AccountsEffect::Invoke(row.clone()));
            ready.pending = Some(row);
        }
        AccountsInput::AskRemove(row) if idle => {
            ready.confirming = ready.pane.owner(&row).and_then(|account| {
                account
                    .remove
                    .as_ref()
                    .filter(|(path, _)| *path == row.path)
                    .map(|_| RemovalQuestion {
                        ask: Ask::Remove,
                        row,
                        account: account.label.clone(),
                        files: if account.sync.iter().any(|sync| sync.on) {
                            LocalFiles::Kept
                        } else {
                            LocalFiles::None
                        },
                    })
            });
        }
        AccountsInput::AskSignOut(row) if idle => {
            ready.confirming = ready.pane.owner(&row).and_then(|account| {
                account
                    .sign_out
                    .as_ref()
                    .filter(|(path, _)| *path == row.path)
                    .map(|_| RemovalQuestion {
                        ask: Ask::SignOut,
                        row,
                        account: account.label.clone(),
                        files: LocalFiles::None,
                    })
            });
        }
        AccountsInput::Keep => ready.confirming = None,
        AccountsInput::Remove if idle => {
            if let Some(asked) = ready.confirming.take() {
                effects.push(AccountsEffect::Invoke(asked.row.clone()));
                ready.pending = Some(asked.row);
            }
        }
        AccountsInput::ClientEdited { row, text } if idle => {
            let change = ready
                .pane
                .clients
                .iter_mut()
                .find(|client| client.module == row.module && client.path == row.path)
                .map_or(ClientChange::Nothing, |client| client.edited(text));
            match change {
                ClientChange::Nothing => {}
                ClientChange::KeyFirst => {
                    effects.push(AccountsEffect::Tell(SignInClient::KEY_FIRST.to_owned()));
                }
                ClientChange::Key(text) => {
                    effects.push(AccountsEffect::Set {
                        row: row.clone(),
                        value: toml::Value::String(text),
                    });
                    ready.pending = Some(row);
                }
                ClientChange::NewSecret(secret) => {
                    effects.push(AccountsEffect::SetSecret {
                        row: row.clone(),
                        secret,
                    });
                    ready.pending = Some(row);
                }
            }
        }
        AccountsInput::Done(row) => {
            if ready.pending.as_ref() == Some(&row) {
                ready.pending = None;
            }
            effects.push(AccountsEffect::Reload(row.module));
        }
        AccountsInput::Failed { row, why } => {
            if ready.pending.as_ref() == Some(&row) {
                ready.pending = None;
            }
            effects.push(AccountsEffect::Tell(why));
            effects.push(AccountsEffect::Reload(row.module));
        }
        AccountsInput::AddAccount if idle && ready.adding == Adding::Idle => {
            ready.adding = Adding::SheetOpen;
            effects.push(AccountsEffect::OpenAddSheet);
        }
        AccountsInput::AddAiKey if idle && ready.adding == Adding::Idle => {
            ready.adding = Adding::SheetOpen;
            effects.push(AccountsEffect::OpenAiKeySheet);
        }
        AccountsInput::AddEnded(module, end) => {
            ready.adding = Adding::Idle;
            effects.extend(end.line().map(|line| AccountsEffect::Tell(line.to_owned())));
            effects.push(AccountsEffect::Reload(module));
            if let AddEnd::AlreadyThere(key) = end {
                effects.push(AccountsEffect::Show(key));
            }
        }
        AccountsInput::Changed(module) => effects.push(AccountsEffect::Reload(module)),
        AccountsInput::SignedIn { row, end } => {
            let label = ready
                .pane
                .owner(&row)
                .map_or("this account", |account| account.label.as_str());
            effects.push(AccountsEffect::Tell(end.line(label)));
            effects.push(AccountsEffect::Reload(row.module));
        }
        AccountsInput::SignedOut { row, end } => {
            let label = ready
                .pane
                .owner(&row)
                .map_or("this account", |account| account.label.as_str());
            effects.push(AccountsEffect::Tell(end.line(label)));
            if end.settles() {
                effects.push(AccountsEffect::Reload(row.module));
            }
        }
        _ => {}
    }
    (ready, effects)
}

/// Whether `row` is an action that may run on one press: a permission's, Sign in again, or
/// syncd's "Continue as". Removing an account and signing an agent out are asked about first,
/// so they are not one.
fn runs_without_asking(pane: &AccountsPane, row: &ModuleRow) -> bool {
    let continues = pane.sync_add.as_ref().is_some_and(|add| add.row() == *row);
    continues
        || pane
            .accounts
            .iter()
            .filter(|a| a.module == row.module)
            .any(|account| {
                account.permissions.iter().any(|p| p.path == row.path)
                    || account
                        .reauth
                        .as_ref()
                        .is_some_and(|(path, _)| *path == row.path)
            })
}
