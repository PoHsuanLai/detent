//! What a pane has to tell the person, as facts. The window turns each into a sentence; the client
//! never words a line that is not the service's own.

use detent_model::{AccountsNotice, AddEnd, AgentWord, ServerRefusal, SyncNowEnd, Way};

/// Something that ended, or went wrong, that the person is told.
#[derive(Debug, Clone, PartialEq)]
pub enum Notice {
    /// A service's own words.
    Words(String),
    /// A change to the computers was refused.
    ComputerRefused(ServerRefusal),
    /// A coding agent cannot be chosen now, or Automatic's list takes it only by choosing it.
    AgentBlocked(AgentWord),
    /// A join of a network ended without the network.
    CouldNotJoin,
    /// The assistant did not come back up after a restart.
    RestartFailed,
    /// The module refused a change to Automatic's list.
    PoolRefused,
    /// The files of a held deletion could not be removed.
    HoldNotRemoved,
    /// The Accounts pane has something to say.
    Account(AccountsNotice),
    /// An add sheet ended.
    AddEnded(AddEnd),
    /// The sync service answered "Sync now".
    SyncNow(SyncNowEnd),
    /// Sharing is set up (or taken away) the way the switch was turned: the setting may be written.
    Recorded(Way),
}
