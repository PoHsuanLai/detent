//! What the Software Update page knows: where the update flow stands, as the update service
//! reports it, parsed once where it enters.

use super::limits::{kind_of, notes_of, problem_of, version_of};
use crate::picker::Absence;

/// Where the update flow stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatePhase {
    /// Nothing to do.
    Idle,
    /// Asking the release site what is newest.
    Checking,
    /// A newer version waits for the person.
    Available,
    /// Downloading it.
    Downloading,
    /// Downloaded and checked, about to be installed.
    Ready,
    /// Switching to it.
    Installing,
    /// Installed; a new login starts it.
    Installed,
    /// The last thing tried did not work.
    Failed,
}

impl UpdatePhase {
    /// The phase the service's word names; `None` for a word this window does not know.
    pub fn of_word(word: &str) -> Option<UpdatePhase> {
        Some(match word {
            "idle" => UpdatePhase::Idle,
            "checking" => UpdatePhase::Checking,
            "available" => UpdatePhase::Available,
            "downloading" => UpdatePhase::Downloading,
            "ready" => UpdatePhase::Ready,
            "installing" => UpdatePhase::Installing,
            "installed" => UpdatePhase::Installed,
            "failed" => UpdatePhase::Failed,
            _ => return None,
        })
    }

    /// Whether the service is working, so a new request would be refused.
    pub fn is_busy(self) -> bool {
        matches!(
            self,
            UpdatePhase::Checking
                | UpdatePhase::Downloading
                | UpdatePhase::Ready
                | UpdatePhase::Installing
        )
    }
}

/// A size in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ByteCount(pub u64);

/// A share of a whole in thousandths: 1000 is all of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Permille(pub u32);

/// A moment, in seconds since 1970.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixSeconds(pub i64);

/// What kind of failure the service reports (its `ProblemKind`), for the kinds this window acts on.
/// It is never shown; the person reads the service's own sentence. A word this window does not
/// know, a word that is too long, and no word at all (an older service) are all [`Self::Other`], a
/// plain failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProblemKind {
    /// An install named a version other than the one on offer: the same thing the refusal
    /// `org.quire.Updates1.Error.VersionChanged` says, found by reading instead.
    VersionChanged,
    /// Any other failure, or none.
    #[default]
    Other,
}

impl ProblemKind {
    /// The kind the service's word names.
    pub fn of_word(word: &str) -> ProblemKind {
        match word {
            "version-changed" => ProblemKind::VersionChanged,
            _ => ProblemKind::Other,
        }
    }
}

/// A version the service offered, as the person reads it: a short run of version characters.
/// It is only made by [`UpdateVersion::parse`], so holding one means it passed the bounds, and it
/// is the one thing an install may name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateVersion(String);

impl UpdateVersion {
    /// `text` as a version; `None` when it is empty, long, or has other than version characters.
    pub fn parse(text: &str) -> Option<UpdateVersion> {
        version_of(text).map(UpdateVersion)
    }

    /// The version as the person reads it: "0.4.0".
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UpdateVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A newer version on offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOffer {
    /// Its version, as the person reads it.
    pub version: UpdateVersion,
    /// What changed in it; empty when the release says nothing.
    pub notes: String,
    /// How much there is to download.
    pub size: ByteCount,
}

/// The update service's standing: everything the page draws comes from here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateStatus {
    /// Where the flow is.
    pub phase: UpdatePhase,
    /// The version that is installed, when the service knows it.
    pub current: Option<String>,
    /// The newer version, when there is one.
    pub offer: Option<UpdateOffer>,
    /// How much of the download is done.
    pub progress: Permille,
    /// When the last check finished; `None` for never.
    pub last_checked: Option<UnixSeconds>,
    /// What went wrong, in plain words, when something did.
    pub problem: Option<String>,
    /// What kind of failure it was; [`ProblemKind::Other`] when nothing went wrong.
    pub problem_kind: ProblemKind,
}

/// The service's properties as they come on the wire (`org.quire.Updates1`): what a read of them
/// gives, before it is parsed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateWire {
    /// `State`.
    pub state: String,
    /// `Current`.
    pub current: String,
    /// `Available`; empty for none.
    pub available: String,
    /// `Notes`.
    pub notes: String,
    /// `Size`, in bytes.
    pub size: u64,
    /// `Progress`, in thousandths.
    pub progress: u32,
    /// `LastChecked`, in Unix seconds; 0 for never.
    pub last_checked: i64,
    /// `Problem`; empty for none.
    pub problem: String,
    /// `ProblemKind`; empty for none, and empty when the service is too old to have it.
    pub problem_kind: String,
}

fn some_text(text: String) -> Option<String> {
    Some(text).filter(|text| !text.is_empty())
}

impl UpdateStatus {
    /// The status the service's properties describe; the service did not make sense when one of
    /// them is a state this window has no word for.
    pub fn of_wire(wire: UpdateWire) -> Result<UpdateStatus, Absence> {
        let phase = UpdatePhase::of_word(&wire.state).ok_or_else(|| {
            Absence::Failed(format!("the update service is in a state {:?}", wire.state))
        })?;
        let odd = |what: &str| Absence::Failed(format!("the update service sent an odd {what}"));
        let current = match some_text(wire.current) {
            Some(text) => Some(version_of(&text).ok_or_else(|| odd("version"))?),
            None => None,
        };
        let offer = match some_text(wire.available) {
            Some(text) => Some(UpdateOffer {
                version: UpdateVersion::parse(&text).ok_or_else(|| odd("version on offer"))?,
                notes: notes_of(&wire.notes),
                size: ByteCount(wire.size),
            }),
            None => None,
        };
        Ok(UpdateStatus {
            phase,
            current,
            offer,
            progress: Permille(wire.progress.min(1000)),
            last_checked: Some(wire.last_checked)
                .filter(|seconds| *seconds > 0)
                .map(UnixSeconds),
            problem: some_text(wire.problem).map(|text| problem_of(&text)),
            problem_kind: kind_of(&wire.problem_kind),
        })
    }
}
