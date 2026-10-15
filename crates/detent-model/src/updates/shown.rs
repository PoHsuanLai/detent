//! What the Software Update page shows for a state: the words, the progress bar, the buttons and
//! the notes sheet, worked out once and drawn as they are.

use super::model::{ByteCount, Permille, UnixSeconds, UpdateOffer, UpdatePhase, UpdateStatus};
use super::step::{NotesState, UpdatesInput, UpdatesReady, UpdatesState};
use super::words::{age_words, size_words};
use crate::picker::Absence;

/// What the page says when nothing serves updates.
pub const UPDATES_NOT_AVAILABLE: &str = "Software updates are not available on this computer.";

/// What it says when the service is there and did not make sense.
const NOT_ANSWERING: &str = "Software updates did not answer. Try again in a moment.";

/// What the daily check is, as the page tells it.
const AUTOMATIC: &str = "Quire checks once a day and asks before it installs anything.";

/// Said when a failure came with no words.
const DID_NOT_FINISH: &str = "The update did not finish. Try again in a moment.";

/// Said when a release came with no notes.
const NO_NOTES: &str = "No release notes came with this update.";

/// Whether a button is waiting for something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonActivity {
    /// It is waiting: it shows a spinner and takes no press.
    Busy,
    /// It is not.
    Still,
}

/// A button of the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateButton {
    /// Look for a newer version.
    Check(ButtonActivity),
    /// Read what the newer version changes.
    MoreInfo,
    /// Install the newer version.
    UpdateNow,
    /// Do again what did not work.
    TryAgain,
    /// Leave the session, so the new version starts.
    LogOut,
}

impl UpdateButton {
    /// The words on the button.
    pub fn label(self) -> &'static str {
        match self {
            UpdateButton::Check(_) => "Check for Updates",
            UpdateButton::MoreInfo => "More Info\u{2026}",
            UpdateButton::UpdateNow => "Update Now",
            UpdateButton::TryAgain => "Try Again",
            UpdateButton::LogOut => "Log Out\u{2026}",
        }
    }

    /// What pressing it tells the page.
    pub fn press(self) -> UpdatesInput {
        match self {
            UpdateButton::Check(_) => UpdatesInput::Check,
            UpdateButton::MoreInfo => UpdatesInput::MoreInfo,
            UpdateButton::UpdateNow => UpdatesInput::UpdateNow,
            UpdateButton::TryAgain => UpdatesInput::TryAgain,
            UpdateButton::LogOut => UpdatesInput::LogOut,
        }
    }
}

/// The bar under the title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateMeter {
    /// There is none.
    None,
    /// How far a download has come, with "12 MB of 45 MB" when the size is known.
    Known {
        /// The share done.
        share: Permille,
        /// How much of how much, when the total is known.
        words: Option<String>,
    },
    /// Work with no end in sight.
    Pending,
}

/// The release notes in their sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotesSheet {
    /// The version the notes are for.
    pub title: String,
    /// The notes.
    pub body: String,
}

/// A page whose service answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatesPage {
    /// The version the row is about: "Quire 0.3.1".
    pub title: String,
    /// What is true of it, under the title.
    pub detail: String,
    /// The bar, if the work has one.
    pub meter: UpdateMeter,
    /// The buttons, the one to press last.
    pub buttons: Vec<UpdateButton>,
    /// When the last check was, when it is worth saying.
    pub checked: Option<String>,
    /// What the daily check is.
    pub automatic: &'static str,
    /// The notes, while they are up.
    pub notes: Option<NotesSheet>,
}

/// What the page draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatesShown {
    /// Still looking: nothing yet.
    Looking,
    /// No updates here, in these words.
    Unavailable(&'static str),
    /// The service answered.
    Page(UpdatesPage),
}

/// What the page draws for `state`, `now` being the time the "last checked" line counts from.
pub fn updates_shown(state: &UpdatesState, now: UnixSeconds) -> UpdatesShown {
    match state {
        UpdatesState::Looking => UpdatesShown::Looking,
        UpdatesState::Unavailable(Absence::Failed(_)) => UpdatesShown::Unavailable(NOT_ANSWERING),
        UpdatesState::Unavailable(_) => UpdatesShown::Unavailable(UPDATES_NOT_AVAILABLE),
        UpdatesState::Ready(ready) => UpdatesShown::Page(page(ready, now)),
    }
}

fn named(version: Option<&str>) -> String {
    version.map_or_else(|| "Quire".to_owned(), |version| format!("Quire {version}"))
}

fn checked_line(status: &UpdateStatus, now: UnixSeconds) -> String {
    status.last_checked.map_or_else(
        || "Not checked yet".to_owned(),
        |then| format!("Last checked {}", age_words(now, then)),
    )
}

/// The row of a state that has nothing to add: `title` and `detail`.
fn plain(title: String, detail: &str) -> UpdatesPage {
    UpdatesPage {
        title,
        detail: detail.to_owned(),
        meter: UpdateMeter::None,
        buttons: Vec::new(),
        checked: None,
        automatic: AUTOMATIC,
        notes: None,
    }
}

fn page(ready: &UpdatesReady, now: UnixSeconds) -> UpdatesPage {
    let status = &ready.status;
    let current = status.current.as_deref();
    let offered = status.offer.as_ref();
    let newer = named(offered.map(|offer| offer.version.as_str()).or(current));
    let installed = named(current);
    match (status.phase, offered) {
        (UpdatePhase::Available, Some(offer)) => available(ready, offer, now),
        (UpdatePhase::Idle | UpdatePhase::Available, _) => UpdatesPage {
            buttons: vec![UpdateButton::Check(ButtonActivity::Still)],
            checked: Some(checked_line(status, now)),
            ..plain(installed, "Your computer is up to date")
        },
        (UpdatePhase::Checking, _) => UpdatesPage {
            buttons: vec![UpdateButton::Check(ButtonActivity::Busy)],
            ..plain(installed, "Checking for updates\u{2026}")
        },
        (UpdatePhase::Downloading, _) => UpdatesPage {
            meter: UpdateMeter::Known {
                share: status.progress,
                words: offered.and_then(|offer| done_of(offer.size, status.progress)),
            },
            ..plain(newer, "Downloading\u{2026}")
        },
        (UpdatePhase::Ready, _) => UpdatesPage {
            meter: UpdateMeter::Pending,
            ..plain(newer, "Preparing to install\u{2026}")
        },
        (UpdatePhase::Installing, _) => UpdatesPage {
            meter: UpdateMeter::Pending,
            ..plain(newer, "Installing\u{2026}")
        },
        (UpdatePhase::Installed, _) => UpdatesPage {
            buttons: vec![UpdateButton::LogOut],
            ..plain(installed, "Log out to finish the update")
        },
        (UpdatePhase::Failed, _) => UpdatesPage {
            buttons: vec![UpdateButton::TryAgain],
            checked: Some(checked_line(status, now)),
            ..plain(newer, status.problem.as_deref().unwrap_or(DID_NOT_FINISH))
        },
    }
}

fn available(ready: &UpdatesReady, offer: &UpdateOffer, now: UnixSeconds) -> UpdatesPage {
    let detail = match offer.size {
        ByteCount(0) => "Update available".to_owned(),
        size => format!("Update available \u{b7} {}", size_words(size)),
    };
    let notes = match ready.info {
        NotesState::Open => Some(NotesSheet {
            title: named(Some(&offer.version)),
            body: if offer.notes.trim().is_empty() {
                NO_NOTES.to_owned()
            } else {
                offer.notes.clone()
            },
        }),
        NotesState::Closed => None,
    };
    UpdatesPage {
        buttons: vec![UpdateButton::MoreInfo, UpdateButton::UpdateNow],
        checked: Some(checked_line(&ready.status, now)),
        notes,
        ..plain(named(Some(&offer.version)), &detail)
    }
}

/// "12.3 MB of 45.6 MB" for a download of `size` that is `progress` done; nothing when the size
/// is not known.
fn done_of(size: ByteCount, progress: Permille) -> Option<String> {
    (size.0 > 0).then(|| {
        let done = u128::from(size.0) * u128::from(progress.0) / 1000;
        let done = ByteCount(u64::try_from(done).unwrap_or(size.0));
        format!("{} of {}", size_words(done), size_words(size))
    })
}
