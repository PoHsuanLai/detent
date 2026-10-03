//! The Settings window: a search field and the page list in a sidebar, the selected page on the
//! right, drawn only with quire's components. Values are read and written through
//! `detent-files`; the programs' own watches apply them.

mod advanced_view;
mod app;
mod controls;
mod env;
mod files;
mod icons;
mod inbox;
mod key_row;
mod page_view;
mod section_view;
mod sidebar;
mod view;
mod window;

pub use app::App;
pub use env::{Env, Opening};
pub use inbox::{Inbox, Notice, Outbox, inbox};

/// The app's own layout stylesheet: tokens only.
pub const STYLE: &str = include_str!("style.css");
