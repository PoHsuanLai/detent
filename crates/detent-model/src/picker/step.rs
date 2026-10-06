//! The picker's one transition: `(state, input) -> (state, effects)`.

use super::model::{Absence, Applying, Described, PickerState, Ready};
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
        PickerInput::Chose { path, word } if ready.applying.is_none() => {
            if ready.row_mut(&path).is_some_and(|row| row.value != word) {
                effects.push(PickerEffect::Set {
                    path: path.clone(),
                    word: word.clone(),
                });
                ready.applying = Some(Applying { path, word });
            }
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
        PickerInput::Chose { .. } | PickerInput::Found(_) => {}
    }
    (PickerState::Ready(ready), effects)
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
