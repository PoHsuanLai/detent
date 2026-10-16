//! The picker's one transition: `(state, input) -> (state, effects)`.

use super::absence::Absence;
use super::choice::{Choice, Gate};
use super::local::{LOCAL_ITEM, LocalAsk};
use super::model::{Applying, AwaitingAccount, Described, PickerState, Ready};
use super::pool::PoolWrite;
use crate::accounts::AddEnd;
use ds_settings::schema::KeyPath;

/// What the person is told when the module refuses a change to Automatic's list.
const POOL_REFUSED: &str = "Automatic could not use that model, so its list is unchanged.";

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
    /// The person chose the engine `engine` (a [`super::LocalEngine`]'s id) for Local in the row
    /// at `path`.
    Engine {
        /// The row.
        path: KeyPath,
        /// The engine's id.
        engine: String,
    },
    /// The person asked to add an account that runs models on this computer, from the row at
    /// `path`.
    SetUpEngine(KeyPath),
    /// The person turned the model `word` in or out of the pool at `path`.
    PoolToggled {
        /// The pool.
        path: KeyPath,
        /// The model.
        word: String,
    },
    /// The module took the write to the pool at `path`.
    PoolApplied(KeyPath),
    /// The module refused the write to the pool at `path`.
    PoolRefused(KeyPath),
    /// The module says the pool at `path` now holds `words`.
    PoolChanged {
        /// The pool.
        path: KeyPath,
        /// Its models.
        words: Vec<String>,
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
    /// Call `Set(path, words)` on the module's pool.
    SetPool {
        /// The pool.
        path: KeyPath,
        /// The models it is to hold.
        words: Vec<String>,
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
        PickerInput::Engine { path, engine } if ready.is_idle() => {
            choose_engine(&mut ready, &path, engine);
        }
        PickerInput::SetUpEngine(path) if ready.is_idle() => {
            ready.awaiting = Some(AwaitingAccount { path, word: None });
            effects.push(PickerEffect::AddAccount);
        }
        PickerInput::PoolToggled { path, word } if ready.is_idle() => {
            effects.extend(toggle_pool(&mut ready, path, word));
        }
        PickerInput::PoolApplied(path) => {
            if let Some(write) = take_pooling(&mut ready, &path) {
                ready.set_pool(&path, &write.words);
            }
        }
        PickerInput::PoolRefused(path) => {
            take_pooling(&mut ready, &path);
            effects.push(PickerEffect::Tell(POOL_REFUSED.to_owned()));
        }
        PickerInput::PoolChanged { path, words } => ready.set_pool(&path, &words),
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
        | PickerInput::Engine { .. }
        | PickerInput::SetUpEngine(_)
        | PickerInput::PoolToggled { .. }
        | PickerInput::Found(_)
        | PickerInput::AccountSheetEnded(_)
        | PickerInput::Reread(_) => {}
    }
    (PickerState::Ready(ready), effects)
}

/// What picking `word` in the row at `path` asks: the set, the add-account sheet (the pick then
/// waits for it), the reason the word cannot be picked, or nothing when the row already has it.
/// Picking Local sets nothing: it opens Local's questions, and a model answers them.
fn choose(ready: &mut Ready, path: KeyPath, word: String) -> Option<PickerEffect> {
    if word == LOCAL_ITEM {
        open_local(ready, &path);
        return None;
    }
    let row = ready.row(&path)?;
    let unchanged = row.value == word;
    match row.gate(&word) {
        Gate::NeedsAccount => {
            ready.awaiting = Some(AwaitingAccount {
                path,
                word: Some(word),
            });
            Some(PickerEffect::AddAccount)
        }
        Gate::Blocked(reason) => Some(PickerEffect::Tell(reason)),
        Gate::Open if unchanged => {
            ready.row_mut(&path)?.local = LocalAsk::Closed;
            None
        }
        Gate::Open => {
            ready.row_mut(&path)?.local = LocalAsk::Closed;
            ready.applying = Some(Applying {
                path: path.clone(),
                word: word.clone(),
            });
            Some(PickerEffect::Set { path, word })
        }
    }
}

/// The pick that waited on an account, once the rows are read again: made if its model can be
/// picked now, dropped if the account did not reach it. An engine asked for reopens Local.
fn pick_again(ready: &mut Ready, pick: AwaitingAccount) -> Option<PickerEffect> {
    let AwaitingAccount { path, word } = pick;
    let Some(word) = word else {
        open_local(ready, &path);
        return None;
    };
    let open = ready
        .row(&path)
        .is_some_and(|row| row.gate(&word) == Gate::Open);
    if open {
        choose(ready, path, word)
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

/// Local was picked in the row at `path`: its questions open, and the stored word stays until a
/// model is chosen. A row that already runs a local model keeps it.
fn open_local(ready: &mut Ready, path: &KeyPath) {
    if let Some(row) = ready.row_mut(path)
        && !matches!(row.choice(), Choice::Local { .. })
    {
        row.local = LocalAsk::Open { engine: None };
    }
}

/// The engine `id` was chosen for Local in the row at `path`; one the row does not list, or
/// the one it already runs, changes nothing.
fn choose_engine(ready: &mut Ready, path: &KeyPath, id: String) {
    let Some(row) = ready.row_mut(path) else {
        return;
    };
    let listed = row.engines().iter().any(|engine| engine.id == id);
    let current = matches!(row.choice(), Choice::Local { engine: Some(own), .. } if own == id);
    if listed && !current {
        row.local = LocalAsk::Open { engine: Some(id) };
    }
}

/// What turning `word` in or out of the pool at `path` asks: the write, or the reason the model
/// cannot be added.
fn toggle_pool(ready: &mut Ready, path: KeyPath, word: String) -> Option<PickerEffect> {
    let pool = ready.pool_at(&path)?;
    let member = pool.chosen.contains(&word);
    let words = pool.toggled(&word);
    if let (Gate::Blocked(reason), false) = (pool.gate(&word), member) {
        return Some(PickerEffect::Tell(reason));
    }
    ready.pooling = Some(PoolWrite {
        path: path.clone(),
        words: words.clone(),
    });
    Some(PickerEffect::SetPool { path, words })
}

/// The pool write in flight, if it is the one at `path`.
fn take_pooling(ready: &mut Ready, path: &KeyPath) -> Option<PoolWrite> {
    match ready.pooling.take() {
        Some(write) if &write.path == path => Some(write),
        other => {
            ready.pooling = other;
            None
        }
    }
}

fn set_value(ready: &mut Ready, path: &KeyPath, word: String) {
    if let Some(row) = ready.row_mut(path) {
        row.value = word;
        row.local = LocalAsk::Closed;
    }
}
