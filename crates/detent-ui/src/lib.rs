//! The Settings window: a search field and the page list in a sidebar, the selected page on the
//! right, drawn only with quire's components. Values are read and written through
//! `detent-files`; the programs' own watches apply them.

mod accounts;
mod advanced_view;
mod app;
mod assistant_model;
mod assistants;
mod computers;
mod controls;
mod engines;
mod env;
mod files;
mod first_run;
mod icons;
mod inbox;
mod inference;
mod key_row;
mod lender;
mod live;
mod marks;
mod memory;
mod page_view;
mod section_view;
mod sharing;
mod shortcuts;
mod sidebar;
mod spaces;
mod updates;
mod view;
mod window;

pub use accounts::read_accounts;
pub use app::App;
pub use env::{Env, Opening};
pub use first_run::FirstRunApp;
pub use inbox::{Inbox, Notice, Outbox, inbox};
pub use inference::ReasonFeed;
pub use lender::Lender;
pub use live::BusSource;
pub use marks::ProviderMarks;
pub use updates::{Departure, LogOut, UpdateTicks, UpdateWork, Updater, UpdatesLink};

/// The app's own layout stylesheet: tokens only.
pub const STYLE: &str = include_str!("style.css");
