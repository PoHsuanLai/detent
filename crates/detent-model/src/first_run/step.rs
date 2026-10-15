//! The first-run window's one transition: `(state, input) -> (state, effects)`.

use super::model::{FirstRun, LocalOnly, Stage};

/// What happened to the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirstRunInput {
    /// The person set whether AI stays on this computer.
    Kept(LocalOnly),
    /// The person pressed Done.
    Finish,
    /// Everything chosen is written.
    Saved,
    /// Writing failed, and why.
    Failed(String),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirstRunEffect {
    /// Write `ai.local_only`, the same write the Intelligence page makes. Answer with `Saved` or
    /// `Failed`.
    Save {
        /// The value of `ai.local_only`.
        local_only: LocalOnly,
    },
    /// Tell the person something went wrong.
    Tell(String),
    /// Remember that the first run happened, and close the window.
    Close,
}

/// The state after `input`, and what to do about it.
pub fn first_run_step(
    mut state: FirstRun,
    input: FirstRunInput,
) -> (FirstRun, Vec<FirstRunEffect>) {
    let mut effects = Vec::new();
    match (state.stage, input) {
        (Stage::Asking, FirstRunInput::Kept(local_only)) => state.local_only = local_only,
        (Stage::Asking, FirstRunInput::Finish) => {
            state.stage = Stage::Saving;
            effects.push(FirstRunEffect::Save {
                local_only: state.local_only,
            });
        }
        (Stage::Saving, FirstRunInput::Saved) => {
            state.stage = Stage::Done;
            effects.push(FirstRunEffect::Close);
        }
        (Stage::Saving, FirstRunInput::Failed(why)) => {
            state.stage = Stage::Asking;
            effects.push(FirstRunEffect::Tell(why));
        }
        _ => {}
    }
    (state, effects)
}
