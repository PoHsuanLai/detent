//! Every call detent makes to another process runs under a deadline (audit DETENT-07), so a
//! daemon that goes quiet, a stuck bus socket or a service waiting on a prompt costs one failed
//! step and a sentence, never a pane or a window that waits for ever.
//!
//! The deadlines are quire's named ones (`Deadline::READ`, `ACTION`, `PROMPT`); a call picks the
//! kind of wait it is and never writes a bare duration. Live-module calls (`LiveClient`) carry
//! theirs inside; this is for the calls detent makes itself.

use ds_peer::within;
use std::future::Future;

pub use ds_peer::Deadline;

/// Who is waited on, as a sentence names them ("The account service is not answering").
pub mod peer {
    /// The bus itself: connecting to it, asking who owns a name.
    pub const DESKTOP: &str = "The desktop";
    /// accountd.
    pub const ACCOUNTS: &str = "The account service";
    /// The assistant (inferd).
    pub const ASSISTANT: &str = "The assistant";
    /// The sync service.
    pub const SYNC: &str = "The sync service";
    /// Memory.
    pub const MEMORY: &str = "Memory";
    /// The screen-sharing portal.
    pub const PORTAL: &str = "The screen sharing service";
    /// The update service.
    pub const UPDATES: &str = "Software Update";
    /// The network service behind Wi-Fi.
    pub const NETWORK: &str = "Wi-Fi";
    /// The Bluetooth service.
    pub const BLUETOOTH: &str = "Bluetooth";
    /// The desktop's display management.
    pub const DISPLAYS: &str = "Displays";
    /// The person's own service manager.
    pub const SERVICES: &str = "This computer's services";
}

/// `work` if it finishes inside `deadline`; `None` when `peer` stays silent. For a call whose
/// failure is already one answer (not reachable, not done).
pub async fn answered<T>(
    deadline: Deadline,
    peer: &'static str,
    work: impl Future<Output = T>,
) -> Option<T> {
    within(deadline, peer, work).await.ok()
}
