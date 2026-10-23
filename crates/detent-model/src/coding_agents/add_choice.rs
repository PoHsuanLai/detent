//! What the Add sheet shows: the assistants it can offer, whether the list of them is still being
//! fetched, and how far Add has come. The words of the failures are here too. Pure.

use super::add::FoundAgent;

/// Said on the Add sheet while the list of coding assistants is fetched.
pub const LOADING: &str = "Looking for coding assistants\u{2026}";

/// Said on the Add sheet when the list of coding assistants cannot be fetched.
pub const LIST_FAILED: &str = "The list of coding assistants can\u{2019}t be fetched right now.";

/// Said under the Add sheet when an assistant cannot be downloaded and nothing says why.
pub const DOWNLOAD_FAILED: &str = "That coding assistant couldn\u{2019}t be downloaded.";

/// Whether an assistant is on this computer already.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    /// Installed: Add only lists it.
    Installed,
    /// Not installed: Add downloads it first.
    Online,
}

impl Presence {
    /// The mark beside an assistant's name; none for one that is not installed.
    pub fn mark(self) -> Option<&'static str> {
        match self {
            Presence::Installed => Some("Installed"),
            Presence::Online => None,
        }
    }
}

/// An assistant the Add sheet offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// The id it is installed and listed under.
    pub id: String,
    /// What the person reads.
    pub name: String,
    /// One line about it; may be empty.
    pub description: String,
    /// The version Add lists (and downloads, when it is not installed).
    pub version: String,
    /// Whether it is installed.
    pub presence: Presence,
}

impl Candidate {
    /// An assistant found in the installed folder, which says nothing of it but its id.
    pub fn installed(found: &FoundAgent) -> Self {
        Self {
            id: found.id.clone(),
            name: found.name(),
            description: String::new(),
            version: found.version.clone(),
            presence: Presence::Installed,
        }
    }
}

/// How far the list of assistants to choose from has come.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Listing {
    /// Being fetched.
    Loading,
    /// There is nothing to fetch it with: only the installed ones are offered.
    Offline,
    /// Fetched.
    Listed,
    /// Not fetched, for this reason in words; the installed ones are offered.
    Failed(String),
}

/// How far Add has come.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Waiting for Add.
    Idle,
    /// Download asked for, nothing said yet.
    Starting,
    /// The download has begun.
    Downloading,
}

/// What the Add button can do now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddButton {
    /// Add can be pressed.
    Ready,
    /// A download is under way.
    Busy,
    /// There is nothing to add yet.
    Off,
}

/// The first line of `text` that says something, without the spaces around it.
fn first_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

/// The sentence for a list that could not be fetched, given what the downloader said (may be empty).
pub fn list_failure(said: &str) -> String {
    match first_line(said) {
        Some(reason) => format!("{LIST_FAILED} {reason}"),
        None => LIST_FAILED.to_owned(),
    }
}

/// The sentence for a download that failed, given what the downloader said (may be empty).
pub fn download_failure(said: &str) -> String {
    first_line(said).unwrap_or(DOWNLOAD_FAILED).to_owned()
}
