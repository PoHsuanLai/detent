//! The Software Update page's one transition: `(state, input) -> (state, effects)`. A press is
//! answered only when the standing allows it, so the window never asks the service for what it
//! would refuse.

use super::model::{UpdatePhase, UpdateStatus};
use crate::picker::Absence;

/// Whether the release notes are up in a sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotesState {
    /// They are not.
    #[default]
    Closed,
    /// They are.
    Open,
}

/// A service that answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatesReady {
    /// What the service says.
    pub status: UpdateStatus,
    /// The notes sheet.
    pub info: NotesState,
}

/// Where the page stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UpdatesState {
    /// Looking for the service.
    #[default]
    Looking,
    /// The service cannot be used, and why.
    Unavailable(Absence),
    /// The service answered.
    Ready(UpdatesReady),
}

/// What happened to the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatesInput {
    /// The service was read, or could not be.
    Heard(Result<UpdateStatus, Absence>),
    /// The person pressed Check for Updates.
    Check,
    /// The person pressed Update Now.
    UpdateNow,
    /// The person pressed Try Again.
    TryAgain,
    /// The person pressed More Info.
    MoreInfo,
    /// The person closed the notes.
    CloseInfo,
    /// The person pressed Log Out.
    LogOut,
    /// The service refused a request, in these words.
    Refused(String),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatesEffect {
    /// Ask the service to look for a newer version.
    Check,
    /// Ask the service to install the newer version.
    Install,
    /// Start logging out.
    LogOut,
    /// Tell the person something went wrong.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn updates_step(
    state: UpdatesState,
    input: UpdatesInput,
) -> (UpdatesState, Vec<UpdatesEffect>) {
    match (state, input) {
        (UpdatesState::Ready(ready), UpdatesInput::Heard(Ok(status))) => {
            let info = match (ready.info, status.phase, &status.offer) {
                (NotesState::Open, UpdatePhase::Available, Some(_)) => NotesState::Open,
                _ => NotesState::Closed,
            };
            (UpdatesState::Ready(UpdatesReady { status, info }), vec![])
        }
        (_, UpdatesInput::Heard(Ok(status))) => (
            UpdatesState::Ready(UpdatesReady {
                status,
                info: NotesState::Closed,
            }),
            vec![],
        ),
        (_, UpdatesInput::Heard(Err(absence))) => (UpdatesState::Unavailable(absence), vec![]),
        (UpdatesState::Ready(ready), input) => {
            let (ready, effects) = ready_step(ready, input);
            (UpdatesState::Ready(ready), effects)
        }
        (state, _) => (state, vec![]),
    }
}

fn ready_step(mut ready: UpdatesReady, input: UpdatesInput) -> (UpdatesReady, Vec<UpdatesEffect>) {
    let phase = ready.status.phase;
    let offered = ready.status.offer.is_some();
    let effects = match input {
        UpdatesInput::Check
            if matches!(
                phase,
                UpdatePhase::Idle | UpdatePhase::Available | UpdatePhase::Failed
            ) =>
        {
            vec![UpdatesEffect::Check]
        }
        UpdatesInput::UpdateNow if phase == UpdatePhase::Available && offered => {
            vec![UpdatesEffect::Install]
        }
        UpdatesInput::TryAgain if phase == UpdatePhase::Failed && offered => {
            vec![UpdatesEffect::Install]
        }
        UpdatesInput::TryAgain if phase == UpdatePhase::Failed => vec![UpdatesEffect::Check],
        UpdatesInput::MoreInfo if phase == UpdatePhase::Available && offered => {
            ready.info = NotesState::Open;
            vec![]
        }
        UpdatesInput::CloseInfo => {
            ready.info = NotesState::Closed;
            vec![]
        }
        UpdatesInput::LogOut if phase == UpdatePhase::Installed => vec![UpdatesEffect::LogOut],
        UpdatesInput::Refused(words) => vec![UpdatesEffect::Tell(words)],
        _ => vec![],
    };
    (ready, effects)
}
