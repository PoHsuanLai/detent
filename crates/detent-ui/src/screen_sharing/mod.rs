//! The Privacy page's screen-sharing apps: the apps the portal lets share the screen without
//! asking again, each with Remove. Showing the group starts the portal when it is not running (the
//! portal remembers its apps on disk), and the owner must still hold the portal's role.

mod pane;
mod view;

pub(crate) use pane::ScreenSharingPane;
