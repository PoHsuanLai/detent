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
mod deadline;
#[cfg(test)]
mod deadline_tests;
mod engine_install;
mod engines;
mod env;
mod files;
mod first_run;
mod icons;
mod inbox;
#[cfg(test)]
mod inbox_tests;
mod inference;
mod key_row;
mod lender;
mod live;
mod marks;
mod memory;
mod page_view;
mod privacy;
mod reported;
mod screen_sharing;
mod section_view;
mod sharing;
mod shortcuts;
mod sidebar;
mod spaces;
mod ticks;
#[cfg(test)]
mod ticks_tests;
mod updates;
mod view;
mod window;

pub use accounts::read_accounts;
pub use app::App;
pub use engine_install::{Desk, EngineDesk, Work};
pub use env::{Env, Opening};
pub use first_run::FirstRunApp;
pub use inbox::{Inbox, Notice, Outbox, inbox};
pub use inference::ReasonFeed;
pub use lender::Lender;
pub use live::{BusSource, Vouch};
pub use marks::ProviderMarks;
pub use ticks::{Gone, Ticker, Ticks};
pub use updates::{
    Departure, InstallRefusal, LogOut, UpdateTicks, UpdateWork, Updater, UpdatesLink,
};

/// The app's own layout stylesheet: tokens only.
pub const STYLE: &str = include_str!("style.css");
