//! The Software Update page: the update service's standing, the buttons that move it, and the
//! log-out that finishes an update. The service, the clock and the log-out each come through a
//! handle the environment carries.

mod drive;
mod leave;
mod link;
mod session;
mod view;
mod wire;

pub use leave::{Departure, LogOut};
pub use link::{UpdateTicks, UpdateWork, Updater, UpdatesLink};
pub(crate) use view::SoftwareUpdatePane;
