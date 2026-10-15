//! What the Software Update page knows: where the update flow stands, as the update service
//! reports it, parsed once where it enters.

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

/// A newer version on offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOffer {
    /// Its version, as the person reads it.
    pub version: String,
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
        Ok(UpdateStatus {
            phase,
            current: some_text(wire.current),
            offer: some_text(wire.available).map(|version| UpdateOffer {
                version,
                notes: wire.notes,
                size: ByteCount(wire.size),
            }),
            progress: Permille(wire.progress.min(1000)),
            last_checked: Some(wire.last_checked)
                .filter(|seconds| *seconds > 0)
                .map(UnixSeconds),
            problem: some_text(wire.problem),
        })
    }
}
