//! The bus every service is reached on: which bus to look on (`BusSource`), who vouches for the
//! owner of a service's name, the daemons a pane reads, and the look-for-it-then-read-it steps
//! the panes share. Nothing here starts a daemon unless a call says so.

mod activate;
mod attach;
mod daemon;
mod io;
mod pinned;
mod read;
#[cfg(test)]
mod read_tests;
mod source;
mod system;
mod vouch;
mod watch;

pub use activate::start;
pub use attach::{attach, attach_listening, open_once, read_once};
pub use daemon::Daemon;
pub use io::{
    absence_of, absence_of_bus, connect, next_change, next_owner, open, owner_changes, plain,
};
pub use pinned::{Pinned, inferd, pinned, pinned_starting};
pub use read::words_of;
pub(crate) use read::{Rows, read};
pub use source::BusSource;
pub use system::{is_running, system_connection};
pub use vouch::Vouch;
pub use watch::{every, next, relay, watch_on, watch_service};
