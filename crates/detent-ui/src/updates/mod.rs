//! The Software Update page: the update service's standing, the buttons that move it, and the
//! log-out that finishes an update. The service, the clock and the log-out each come through a
//! handle the environment carries.

mod view;

pub(crate) use view::SoftwareUpdatePane;
