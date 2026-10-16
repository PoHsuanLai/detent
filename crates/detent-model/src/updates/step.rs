//! The Software Update page's one transition: `(state, input) -> (state, effects)`. A press is
//! answered only when the standing allows it, so the window never asks the service for what it
//! would refuse.

use super::model::{UpdatePhase, UpdateStatus, UpdateVersion, VERSION_CHANGED};
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

/// What the page says besides the standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdatesNotice {
    /// Nothing.
    #[default]
    Nothing,
    /// The version on offer is newer than the one the person was looking at, and was not
    /// installed: they look at it before pressing again.
    NewerVersion,
}

/// A service that answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatesReady {
    /// What the service says.
    pub status: UpdateStatus,
    /// The notes sheet.
    pub info: NotesState,
    /// What the page says besides the standing.
    pub notice: UpdatesNotice,
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
    /// The service would not install the version that was named: what it offers is another now
    /// (the caller has read the service again by then).
    VersionChanged,
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatesEffect {
    /// Ask the service to look for a newer version.
    Check,
    /// Ask the service to install this version, the one the page shows.
    Install(UpdateVersion),
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
        (UpdatesState::Ready(before), UpdatesInput::Heard(Ok(status))) => {
            (UpdatesState::Ready(heard(Some(&before), status)), vec![])
        }
        (_, UpdatesInput::Heard(Ok(status))) => (UpdatesState::Ready(heard(None, status)), vec![]),
        (_, UpdatesInput::Heard(Err(absence))) => (UpdatesState::Unavailable(absence), vec![]),
        (UpdatesState::Ready(ready), input) => {
            let (ready, effects) = ready_step(ready, input);
            (UpdatesState::Ready(ready), effects)
        }
        (state, _) => (state, vec![]),
    }
}

/// Whether the page shows an update to take: one waits, and a version is on offer.
fn is_offering(status: &UpdateStatus) -> bool {
    status.phase == UpdatePhase::Available && status.offer.is_some()
}

/// The service's standing after a failure that only says the offer changed under an install: it
/// is an offer again, with the note, since the person has not seen this version.
fn overtaken(status: UpdateStatus) -> (UpdateStatus, UpdatesNotice) {
    let changed = status.phase == UpdatePhase::Failed
        && status.problem.as_deref() == Some(VERSION_CHANGED)
        && status.offer.is_some();
    if changed {
        let status = UpdateStatus {
            phase: UpdatePhase::Available,
            problem: None,
            ..status
        };
        (status, UpdatesNotice::NewerVersion)
    } else {
        (status, UpdatesNotice::Nothing)
    }
}

/// The page after the service was read: the sheet and the note stay while the same kind of offer
/// waits, and go when it does not.
fn heard(before: Option<&UpdatesReady>, status: UpdateStatus) -> UpdatesReady {
    let (status, found) = overtaken(status);
    let offering = is_offering(&status);
    let (info, kept) = before.map_or((NotesState::Closed, UpdatesNotice::Nothing), |ready| {
        (ready.info, ready.notice)
    });
    let info = match info {
        NotesState::Open if offering => NotesState::Open,
        _ => NotesState::Closed,
    };
    let notice = match (found, kept) {
        (UpdatesNotice::Nothing, UpdatesNotice::Nothing) => UpdatesNotice::Nothing,
        _ if offering => UpdatesNotice::NewerVersion,
        _ => UpdatesNotice::Nothing,
    };
    UpdatesReady {
        status,
        info,
        notice,
    }
}

/// The version an install names: the one on offer, when there is one. With none on offer there
/// is nothing to name, so nothing to ask.
fn install_of(status: &UpdateStatus) -> Vec<UpdatesEffect> {
    status
        .offer
        .iter()
        .map(|offer| UpdatesEffect::Install(offer.version.clone()))
        .collect()
}

fn ready_step(mut ready: UpdatesReady, input: UpdatesInput) -> (UpdatesReady, Vec<UpdatesEffect>) {
    let phase = ready.status.phase;
    let effects = match input {
        UpdatesInput::Check
            if matches!(
                phase,
                UpdatePhase::Idle | UpdatePhase::Available | UpdatePhase::Failed
            ) =>
        {
            ready.notice = UpdatesNotice::Nothing;
            vec![UpdatesEffect::Check]
        }
        UpdatesInput::UpdateNow if phase == UpdatePhase::Available => {
            ready.notice = UpdatesNotice::Nothing;
            install_of(&ready.status)
        }
        UpdatesInput::TryAgain if phase == UpdatePhase::Failed => {
            ready.notice = UpdatesNotice::Nothing;
            match install_of(&ready.status) {
                installs if installs.is_empty() => vec![UpdatesEffect::Check],
                installs => installs,
            }
        }
        UpdatesInput::MoreInfo if is_offering(&ready.status) => {
            ready.info = NotesState::Open;
            vec![]
        }
        UpdatesInput::CloseInfo => {
            ready.info = NotesState::Closed;
            vec![]
        }
        UpdatesInput::LogOut if phase == UpdatePhase::Installed => vec![UpdatesEffect::LogOut],
        UpdatesInput::Refused(words) => vec![UpdatesEffect::Tell(words)],
        UpdatesInput::VersionChanged if is_offering(&ready.status) => {
            ready.notice = UpdatesNotice::NewerVersion;
            vec![]
        }
        _ => vec![],
    };
    (ready, effects)
}
