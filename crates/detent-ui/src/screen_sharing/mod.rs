//! The Privacy page's screen-sharing apps: the apps the portal lets share the screen without
//! asking again, each with Remove. Settings looks for the portal and never starts it.

mod io;
mod pane;
mod proxy;
mod view;
mod wire;

pub(crate) use pane::ScreenSharingPane;
