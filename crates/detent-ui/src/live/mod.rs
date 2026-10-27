//! Following a daemon's live settings module (`org.quire.SettingsModule1`) as a pane does: look
//! for it when the pane appears, read its rows, and read again each time it comes back or says a
//! row changed. The calls themselves are `detent-system`'s.

mod follow;

pub(crate) use follow::{Followed, follow};
