//! A daemon's live settings module (`org.quire.SettingsModule1`), as a pane meets it: the bus to
//! find it on, the daemon's name and path, and the look-for-it-then-follow-it loop that inferd's
//! picker and accountd's pane share; and the start of the daemons a page reads when it is shown.

mod activate;
mod bus;
mod daemon;
mod follow;
mod holds;
mod io;
mod read;
#[cfg(test)]
mod read_tests;

pub(crate) use activate::start;
pub use bus::BusSource;
pub(crate) use daemon::Daemon;
pub(crate) use follow::{Followed, follow, open_once, read_once, reread};
pub(crate) use holds::{confirm_discard, follow_holds, hold_now, sync_now};
pub(crate) use io::{connect, next_owner, owner_changes};
pub(crate) use read::words_of;
