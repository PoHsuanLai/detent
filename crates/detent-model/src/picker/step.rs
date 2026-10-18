//! The picker's one transition: `(state, input) -> (state, effects)`.

use super::choice::{Choice, Gate};
use super::local::{LOCAL_ITEM, LocalAsk};
use super::model::{Applying, AwaitingAccount, Busy, Described, PickerState, Ready, TurnOn};
use super::pool::PoolWrite;
use crate::Absence;
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
    /// The person answered whether to turn coding agents on, asked when they picked one.
    TurnOn(TurnOnAnswer),
    /// Coding agents were switched on, or the reason they could not be.
    AgentsTurnedOn(Result<(), String>),
}

/// What the person said to turning coding agents on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnOnAnswer {
    /// Turn them on, and use the one picked.
    Yes,
    /// Leave them off.
    No,
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
    /// Switch coding agents on (`agent.acp.agents`) and answer with `AgentsTurnedOn`.
    TurnOnAgents,
    /// Tell the person something went wrong.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn step(state: PickerState, input: PickerInput) -> (PickerState, Vec<PickerEffect>) {
    match input {
        PickerInput::Found(read) => (
            state.found(
                read,
                |described| Ready::of(&described),
                |_, described| Ready::of(&described),
            ),
            vec![],
        ),
        other => state.inner(|ready| ready_step(ready, other)),
    }
}

fn ready_step(mut ready: Ready, input: PickerInput) -> (Ready, Vec<PickerEffect>) {
    let mut effects = Vec::new();
    match input {
        PickerInput::Chose { path, word } if ready.is_idle() => {
            effects.extend(choose(&mut ready, path, word));
        }
        PickerInput::Engine { path, engine } if ready.is_idle() => {
            choose_engine(&mut ready, &path, engine);
        }
        PickerInput::SetUpEngine(path) if ready.is_idle() => {
            ready.busy = Busy::Awaiting(AwaitingAccount { path, word: None });
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
        PickerInput::AccountSheetEnded(end) if ready.awaiting().is_some() => match end {
            AddEnd::Added => effects.push(PickerEffect::ReadAgain),
            AddEnd::Closed => ready.busy = Busy::Idle,
            other => {
                ready.busy = Busy::Idle;
                effects.extend(other.line().map(|line| PickerEffect::Tell(line.to_owned())));
            }
        },
        PickerInput::Reread(described) if ready.awaiting().is_some() => {
            let awaiting = take_awaiting(&mut ready);
            ready = Ready::of(&described);
            effects.extend(awaiting.and_then(|pick| pick_again(&mut ready, pick)));
        }
        PickerInput::TurnOn(answer) if ready.turning_on().is_some() => match answer {
            TurnOnAnswer::No => ready.busy = Busy::Idle,
            TurnOnAnswer::Yes => effects.push(PickerEffect::TurnOnAgents),
        },
        PickerInput::AgentsTurnedOn(done) if ready.turning_on().is_some() => {
            let asked = take_turning_on(&mut ready);
            match done {
                Ok(()) => {
                    ready.agents_are_on();
                    effects.extend(asked.and_then(|pick| choose(&mut ready, pick.path, pick.word)));
                }
                Err(why) => effects.push(PickerEffect::Tell(why)),
            }
        }
        PickerInput::Chose { .. }
        | PickerInput::TurnOn(_)
        | PickerInput::AgentsTurnedOn(_)
        | PickerInput::Engine { .. }
        | PickerInput::SetUpEngine(_)
        | PickerInput::PoolToggled { .. }
        | PickerInput::Found(_)
        | PickerInput::AccountSheetEnded(_)
        | PickerInput::Reread(_) => {}
    }
    (ready, effects)
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
            ready.busy = Busy::Awaiting(AwaitingAccount {
                path,
                word: Some(word),
            });
            Some(PickerEffect::AddAccount)
        }
        Gate::Blocked(reason) => Some(PickerEffect::Tell(reason)),
        Gate::TurnOn => {
            ready.busy = Busy::TurningOn(TurnOn { path, word });
            None
        }
        Gate::Open if unchanged => {
            ready.row_mut(&path)?.local = LocalAsk::Closed;
            None
        }
        Gate::Open => {
            ready.row_mut(&path)?.local = LocalAsk::Closed;
            ready.busy = Busy::Applying(Applying {
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
    match std::mem::take(&mut ready.busy) {
        Busy::Applying(applying) if &applying.path == path => Some(applying),
        other => {
            ready.busy = other;
            None
        }
    }
}

/// The pick that waits on an account, if that is what the picker waits on.
fn take_awaiting(ready: &mut Ready) -> Option<AwaitingAccount> {
    match std::mem::take(&mut ready.busy) {
        Busy::Awaiting(pick) => Some(pick),
        other => {
            ready.busy = other;
            None
        }
    }
}

/// The pick that waits on the person's yes, if that is what the picker waits on.
fn take_turning_on(ready: &mut Ready) -> Option<TurnOn> {
    match std::mem::take(&mut ready.busy) {
        Busy::TurningOn(asked) => Some(asked),
        other => {
            ready.busy = other;
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
    ready.busy = Busy::Pooling(PoolWrite {
        path: path.clone(),
        words: words.clone(),
    });
    Some(PickerEffect::SetPool { path, words })
}

/// The pool write in flight, if it is the one at `path`.
fn take_pooling(ready: &mut Ready, path: &KeyPath) -> Option<PoolWrite> {
    match std::mem::take(&mut ready.busy) {
        Busy::Pooling(write) if &write.path == path => Some(write),
        other => {
            ready.busy = other;
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
