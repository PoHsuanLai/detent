//! The picker's one transition: `(state, input) -> (state, effects)`.

use super::choice::Gate;
use super::model::{Absence, Applying, AwaitingAccount, Described, PickerState, Ready};
use crate::accounts::AddEnd;
use ds_settings::schema::KeyPath;

/// What happened to the picker.
#[derive(Debug, Clone, PartialEq)]
pub enum PickerInput {
    /// The first look at the module came back.
    Found(Result<Described, Absence>),
    /// The person picked `word` for the row at `path`.
    Chose {
        /// The row.
        path: KeyPath,
        /// The word.
        word: String,
    },
    /// The module took the set at `path`.
    Applied(KeyPath),
    /// The module refused the set at `path`.
    Refused {
        /// The row.
        path: KeyPath,
        /// What the module said.
        why: String,
    },
    /// The module says `path` is now `word` (someone else set it, or the file changed).
    Changed {
        /// The row.
        path: KeyPath,
        /// Its word.
        word: String,
    },
    /// The add-account sheet, opened for a pick, ended.
    AccountSheetEnded(AddEnd),
    /// The module's rows, read again after an account was added.
    Reread(Described),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerEffect {
    /// Call `Set(path, word)` on the module.
    Set {
        /// The row.
        path: KeyPath,
        /// The word.
        word: String,
    },
    /// Open the add-account sheet and answer with `AccountSheetEnded`.
    AddAccount,
    /// Read the module's rows again and answer with `Reread`.
    ReadAgain,
    /// Tell the person something went wrong.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn step(state: PickerState, input: PickerInput) -> (PickerState, Vec<PickerEffect>) {
    match (state, input) {
        (_, PickerInput::Found(Ok(described))) => {
            (PickerState::Ready(Ready::of(&described)), vec![])
        }
        (_, PickerInput::Found(Err(absence))) => (PickerState::Unavailable(absence), vec![]),
        (PickerState::Ready(ready), input) => ready_step(ready, input),
        (state, _) => (state, vec![]),
    }
}

fn ready_step(mut ready: Ready, input: PickerInput) -> (PickerState, Vec<PickerEffect>) {
    let mut effects = Vec::new();
    match input {
        PickerInput::Chose { path, word } if ready.is_idle() => {
            effects.extend(choose(&mut ready, path, word));
        }
        PickerInput::Applied(path) => {
            if let Some(Applying { word, .. }) = take_applying(&mut ready, &path) {
                set_value(&mut ready, &path, word);
            }
        }
        PickerInput::Refused { path, why } => {
            take_applying(&mut ready, &path);
            effects.push(PickerEffect::Tell(why));
        }
        PickerInput::Changed { path, word } => set_value(&mut ready, &path, word),
        PickerInput::AccountSheetEnded(end) if ready.awaiting.is_some() => match end {
            AddEnd::Added => effects.push(PickerEffect::ReadAgain),
            AddEnd::Closed => ready.awaiting = None,
            other => {
                ready.awaiting = None;
                effects.extend(other.line().map(|line| PickerEffect::Tell(line.to_owned())));
            }
        },
        PickerInput::Reread(described) if ready.awaiting.is_some() => {
            let awaiting = ready.awaiting.take();
            ready = Ready::of(&described);
            effects.extend(awaiting.and_then(|pick| pick_again(&mut ready, pick)));
        }
        PickerInput::Chose { .. }
        | PickerInput::Found(_)
        | PickerInput::AccountSheetEnded(_)
        | PickerInput::Reread(_) => {}
    }
    (PickerState::Ready(ready), effects)
}

/// What picking `word` in the row at `path` asks: the set, the add-account sheet (the pick then
/// waits for it), the reason the word cannot be picked, or nothing when the row already has it.
fn choose(ready: &mut Ready, path: KeyPath, word: String) -> Option<PickerEffect> {
    let row = ready.row(&path)?;
    let unchanged = row.value == word;
    match row.gate(&word) {
        Gate::NeedsAccount => {
            ready.awaiting = Some(AwaitingAccount { path, word });
            Some(PickerEffect::AddAccount)
        }
        Gate::Blocked(reason) => Some(PickerEffect::Tell(reason)),
        Gate::Open if unchanged => None,
        Gate::Open => {
            ready.applying = Some(Applying {
                path: path.clone(),
                word: word.clone(),
            });
            Some(PickerEffect::Set { path, word })
        }
    }
}

/// The pick that waited on an account, once the rows are read again: made if its model can be
/// picked now, dropped if the account did not reach it.
fn pick_again(ready: &mut Ready, pick: AwaitingAccount) -> Option<PickerEffect> {
    let open = ready
        .row(&pick.path)
        .is_some_and(|row| row.gate(&pick.word) == Gate::Open);
    if open {
        choose(ready, pick.path, pick.word)
    } else {
        None
    }
}

/// The set in flight, if it is the one at `path`.
fn take_applying(ready: &mut Ready, path: &KeyPath) -> Option<Applying> {
    match ready.applying.take() {
        Some(applying) if &applying.path == path => Some(applying),
        other => {
            ready.applying = other;
            None
        }
    }
}

fn set_value(ready: &mut Ready, path: &KeyPath, word: String) {
    if let Some(row) = ready.row_mut(path) {
        row.value = word;
    }
}
