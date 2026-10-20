//! What Wi-Fi, Bluetooth and Displays share: the system bus the first two live on, a watch on the
//! service behind a page, and the row that says a page has nothing to show.

mod bus;
mod unavailable;
mod watch;

pub(crate) use bus::{is_running, system_connection};
pub(crate) use unavailable::Unavailable;
pub(crate) use watch::{every, next, watch_service};
