//! What the Add sheet shows: the assistants it can offer, whether the list of them is still being
//! fetched, and how far Add has come. Pure.

use super::add::FoundAgent;

/// Whether an assistant is on this computer already.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    /// Installed: Add only lists it.
    Installed,
    /// Not installed: Add downloads it first.
    Online,
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
    /// Not fetched; the installed ones are offered. Holds the first line the downloader said, if it
    /// said any.
    Failed(Option<String>),
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
pub enum AddAction {
    /// Add can be pressed.
    Ready,
    /// A download is under way.
    Busy,
    /// There is nothing to add yet.
    Off,
}

/// The first line of `said` that says something, without the spaces around it.
pub(super) fn said_line(said: &str) -> Option<String> {
    said.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}
