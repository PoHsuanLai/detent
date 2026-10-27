//! The pane's one transition: `(state, input) -> (state, effects)`.

use super::add_account::{AddEnd, Adding};
use super::effect::{AccountsEffect, AccountsNotice};
use super::input::AccountsInput;
use super::model::{
    AccountsPane, AccountsReady, AccountsScreen, AccountsState, Ask, LocalFiles, RemovalQuestion,
    SwitchState,
};
use super::module::{ModuleId, ModuleRow, Sources};
use super::sign_in_client::{ClientChange, ClientText, SignInClient};
use crate::Absence;
use crate::picker::Described;

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
        // The page is a list beside the chosen account's detail: choosing an account shows it.
        AccountsInput::Open(key) | AccountsInput::Select(key)
            if ready.pane.account(&key).is_some() =>
        {
            ready.selected = Some(key.clone());
            ready.screen = AccountsScreen::Detail(key);
        }
        AccountsInput::Back => {
            ready.screen = AccountsScreen::List;
            ready.selected = None;
            ready.confirming = None;
        }
        AccountsInput::Switched { row, on } if idle => {
            effects.extend(switched(&mut ready, row, on));
        }
        AccountsInput::Ran(row) if idle && runs_without_asking(&ready.pane, &row) => {
            effects.push(AccountsEffect::Invoke(row.clone()));
            ready.pending = Some(row);
        }
        AccountsInput::AskRemove(row) if idle => {
            ready.confirming = removal_question(&ready.pane, row, Ask::Remove);
        }
        AccountsInput::AskSignOut(row) if idle => {
            ready.confirming = removal_question(&ready.pane, row, Ask::SignOut);
        }
        AccountsInput::Keep => ready.confirming = None,
        AccountsInput::Remove if idle => {
            if let Some(asked) = ready.confirming.take() {
                effects.push(AccountsEffect::Invoke(asked.row.clone()));
                ready.pending = Some(asked.row);
            }
        }
        AccountsInput::ClientEdited { row, text } if idle => {
            effects.extend(client_edited(&mut ready, row, text));
        }
        AccountsInput::SecretCleared(row) if idle => {
            effects.extend(secret_cleared(&mut ready, row));
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
            effects.push(AccountsEffect::Tell(AccountsNotice::Refused(why)));
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
            if !matches!(end, AddEnd::Added | AddEnd::Closed) {
                effects.push(AccountsEffect::Tell(AccountsNotice::AddEnded(end.clone())));
            }
            effects.push(AccountsEffect::Reload(module));
            if let AddEnd::AlreadyThere(key) = end {
                effects.push(AccountsEffect::Show(key));
            }
        }
        AccountsInput::Changed(module) => effects.push(AccountsEffect::Reload(module)),
        AccountsInput::SignedIn { row, end } => {
            let account = ready.pane.owner(&row).map(|account| account.label.clone());
            effects.push(AccountsEffect::Tell(AccountsNotice::SignedIn {
                end,
                account,
            }));
            effects.push(AccountsEffect::Reload(row.module));
        }
        AccountsInput::SignedOut { row, end } => {
            let account = ready.pane.owner(&row).map(|account| account.label.clone());
            effects.push(AccountsEffect::Tell(AccountsNotice::SignedOut {
                end,
                account,
            }));
            if end.settles() {
                effects.push(AccountsEffect::Reload(row.module));
            }
        }
        _ => {}
    }
    (ready, effects)
}

/// The `Set` a switch asks for, and the pane with the switch already turned: a service, a sync
/// switch and Show in Files are each a path and a state, the same `Set`. None when no switch of
/// the row's module is at the path or it is already in that state.
fn switched(ready: &mut AccountsReady, row: ModuleRow, on: SwitchState) -> Option<AccountsEffect> {
    let (_, held) = ready
        .pane
        .accounts
        .iter_mut()
        .filter(|account| account.module == row.module)
        .flat_map(|account| {
            let services = account.services.iter_mut().map(|s| (&s.path, &mut s.on));
            let sync = account
                .sync
                .iter_mut()
                .chain(account.show_in_files.as_mut());
            services.chain(sync.map(|s| (&s.path, &mut s.on)))
        })
        .find(|(path, held)| **path == row.path && **held != on)?;
    *held = on;
    ready.pending = Some(row.clone());
    Some(AccountsEffect::Set {
        row,
        value: toml::Value::String(on.word().to_owned()),
    })
}

/// The question asked before `ask` runs at `row`; none when the row is not that account's action.
fn removal_question(pane: &AccountsPane, row: ModuleRow, ask: Ask) -> Option<RemovalQuestion> {
    let account = pane.owner(&row)?;
    let action = match ask {
        Ask::Remove => account.remove.as_ref(),
        Ask::SignOut => account.sign_out.as_ref(),
    };
    action.filter(|(path, _)| *path == row.path)?;
    let files = match ask {
        Ask::Remove if account.sync.iter().any(|sync| sync.on.is_on()) => LocalFiles::Kept,
        Ask::Remove | Ask::SignOut => LocalFiles::None,
    };
    Some(RemovalQuestion {
        ask,
        row,
        account: account.label.clone(),
        files,
    })
}

/// What the person typing into a sign-in row asks of the module, if anything; the row waits for
/// the answer.
fn client_edited(
    ready: &mut AccountsReady,
    row: ModuleRow,
    text: ClientText,
) -> Option<AccountsEffect> {
    let change = ready
        .pane
        .clients
        .iter_mut()
        .find(|client| client.module == row.module && client.path == row.path)
        .map_or(ClientChange::Nothing, |client| client.edited(text));
    let effect = match change {
        ClientChange::Nothing => return None,
        ClientChange::KeyFirst => return Some(AccountsEffect::Tell(AccountsNotice::KeyFirst)),
        ClientChange::Key(text) => AccountsEffect::Set {
            row: row.clone(),
            value: toml::Value::String(text),
        },
        ClientChange::NewSecret(secret) => AccountsEffect::SetSecret {
            row: row.clone(),
            secret,
        },
        // Only Clear produces this, and it has its own input.
        ClientChange::ClearSecret => return None,
    };
    ready.pending = Some(row);
    Some(effect)
}

/// What pressing Clear beside a sign-in secret asks of the module, if anything.
fn secret_cleared(ready: &mut AccountsReady, row: ModuleRow) -> Option<AccountsEffect> {
    let change = ready
        .pane
        .clients
        .iter_mut()
        .find(|client| client.module == row.module && client.path == row.path)
        .map_or(ClientChange::Nothing, SignInClient::cleared);
    (change == ClientChange::ClearSecret).then(|| {
        ready.pending = Some(row.clone());
        // The empty text carries no secret, so it travels as a plain value.
        AccountsEffect::Set {
            row,
            value: toml::Value::String(String::new()),
        }
    })
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
