//! What the Software Update page shows for a state, as facts: the standing, the bar, the actions
//! on offer and the notes. The window words them (`detent-ui`'s copy).

use super::model::{ByteCount, Permille, UnixSeconds, UpdateOffer, UpdatePhase, UpdateStatus};
use super::step::{NotesState, UpdatesInput, UpdatesNotice, UpdatesReady, UpdatesState};
use crate::Live;

/// Whether the check is under way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckActivity {
    /// A check is under way: nothing more can be asked of it yet.
    Running,
    /// None is.
    Resting,
}

/// What the person may do on the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Look for a newer version.
    Check(CheckActivity),
    /// Read what the newer version changes.
    MoreInfo,
    /// Install the newer version.
    UpdateNow,
    /// Do again what did not work.
    TryAgain,
    /// Leave the session, so the new version starts.
    LogOut,
}

impl UpdateAction {
    /// What taking the action tells the page.
    pub fn input(self) -> UpdatesInput {
        match self {
            UpdateAction::Check(_) => UpdatesInput::Check,
            UpdateAction::MoreInfo => UpdatesInput::MoreInfo,
            UpdateAction::UpdateNow => UpdatesInput::UpdateNow,
            UpdateAction::TryAgain => UpdatesInput::TryAgain,
            UpdateAction::LogOut => UpdatesInput::LogOut,
        }
    }
}

/// How much of a download is done, of how much.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Downloaded {
    /// What has come.
    pub done: ByteCount,
    /// All of it.
    pub total: ByteCount,
}

/// The bar under the title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateMeter {
    /// There is none.
    None,
    /// How far a download has come, with how much of how much when the size is known.
    Known {
        /// The share done.
        share: Permille,
        /// How much of how much, when the total is known.
        done: Option<Downloaded>,
    },
    /// Work with no end in sight.
    Pending,
}

/// The release notes of a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseNotes {
    /// The version the notes are for.
    pub version: String,
    /// The notes; `None` when the release came with none.
    pub text: Option<String>,
}

/// What is true of the version the row is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStanding {
    /// Nothing newer waits.
    UpToDate,
    /// Looking for a newer version.
    Checking,
    /// A newer version waits, of this size (zero when it is not known).
    Available(ByteCount),
    /// Downloading it.
    Downloading,
    /// Downloaded, about to be installed.
    Preparing,
    /// Switching to it.
    Installing,
    /// Installed; a new login starts it.
    Installed,
    /// The last thing tried did not work, in the service's words when it said.
    Failed(Option<String>),
}

/// When the last check was, when it is worth saying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checked {
    /// It is not said.
    Unsaid,
    /// There has been none.
    Never,
    /// It finished this many seconds ago (a clock a little ahead counts as none).
    Ago(i64),
}

/// A page whose service answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatesPage {
    /// The version the row is about; `None` when no version is known.
    pub version: Option<String>,
    /// What is true of it.
    pub standing: UpdateStanding,
    /// What the page says besides the standing.
    pub notice: UpdatesNotice,
    /// The bar, if the work has one.
    pub meter: UpdateMeter,
    /// The actions, the one to take last.
    pub actions: Vec<UpdateAction>,
    /// When the last check was.
    pub checked: Checked,
    /// The notes, while they are up.
    pub notes: Option<ReleaseNotes>,
}

/// What the page draws: nothing yet, why there is nothing, or the page.
pub type UpdatesShown = Live<UpdatesPage>;

/// What the page draws for `state`, `now` being the time the last check counts from.
pub fn updates_shown(state: &UpdatesState, now: UnixSeconds) -> UpdatesShown {
    state.map(|ready| page(ready, now))
}

fn checked_of(status: &UpdateStatus, now: UnixSeconds) -> Checked {
    status.last_checked.map_or(Checked::Never, |then| {
        Checked::Ago(now.0.saturating_sub(then.0))
    })
}

/// The page of a state that has nothing to add: `version` and `standing`.
fn plain(version: Option<String>, standing: UpdateStanding) -> UpdatesPage {
    UpdatesPage {
        version,
        standing,
        notice: UpdatesNotice::Nothing,
        meter: UpdateMeter::None,
        actions: Vec::new(),
        checked: Checked::Unsaid,
        notes: None,
    }
}

fn page(ready: &UpdatesReady, now: UnixSeconds) -> UpdatesPage {
    let status = &ready.status;
    let current = status.current.clone();
    let offered = status.offer.as_ref();
    let newer = offered
        .map(|offer| offer.version.as_str().to_owned())
        .or_else(|| current.clone());
    match (status.phase, offered) {
        (UpdatePhase::Available, Some(offer)) => available(ready, offer, now),
        (UpdatePhase::Idle | UpdatePhase::Available, _) => UpdatesPage {
            actions: vec![UpdateAction::Check(CheckActivity::Resting)],
            checked: checked_of(status, now),
            ..plain(current, UpdateStanding::UpToDate)
        },
        (UpdatePhase::Checking, _) => UpdatesPage {
            actions: vec![UpdateAction::Check(CheckActivity::Running)],
            ..plain(current, UpdateStanding::Checking)
        },
        (UpdatePhase::Downloading, _) => UpdatesPage {
            meter: UpdateMeter::Known {
                share: status.progress,
                done: offered.and_then(|offer| done_of(offer.size, status.progress)),
            },
            ..plain(newer, UpdateStanding::Downloading)
        },
        (UpdatePhase::Ready, _) => UpdatesPage {
            meter: UpdateMeter::Pending,
            ..plain(newer, UpdateStanding::Preparing)
        },
        (UpdatePhase::Installing, _) => UpdatesPage {
            meter: UpdateMeter::Pending,
            ..plain(newer, UpdateStanding::Installing)
        },
        (UpdatePhase::Installed, _) => UpdatesPage {
            actions: vec![UpdateAction::LogOut],
            ..plain(current, UpdateStanding::Installed)
        },
        (UpdatePhase::Failed, _) => UpdatesPage {
            actions: vec![UpdateAction::TryAgain],
            checked: checked_of(status, now),
            ..plain(newer, UpdateStanding::Failed(status.problem.clone()))
        },
    }
}

fn available(ready: &UpdatesReady, offer: &UpdateOffer, now: UnixSeconds) -> UpdatesPage {
    let version = offer.version.as_str().to_owned();
    let notes = match ready.info {
        NotesState::Open => Some(ReleaseNotes {
            version: version.clone(),
            text: Some(offer.notes.clone()).filter(|text| !text.trim().is_empty()),
        }),
        NotesState::Closed => None,
    };
    UpdatesPage {
        notice: ready.notice,
        actions: vec![UpdateAction::MoreInfo, UpdateAction::UpdateNow],
        checked: checked_of(&ready.status, now),
        notes,
        ..plain(Some(version), UpdateStanding::Available(offer.size))
    }
}

/// How much of a download of `size` is done at `progress`; nothing when the size is not known.
fn done_of(size: ByteCount, progress: Permille) -> Option<Downloaded> {
    (size.0 > 0).then(|| {
        let done = u128::from(size.0) * u128::from(progress.0) / 1000;
        Downloaded {
            done: ByteCount(u64::try_from(done).unwrap_or(size.0)),
            total: size,
        }
    })
}
