//! The Settings window: a search field and the page list in a sidebar, the selected page on the
//! right, drawn only with quire's components. Values are read and written through
//! `detent-files`; the programs' own watches apply them.

mod accounts;
mod advanced_view;
mod app;
mod assistant_model;
mod assistants;
mod bluetooth;
mod chooser;
mod computers;
mod controls;
mod deadline;
#[cfg(test)]
mod deadline_tests;
mod displays;
mod engine_install;
mod engines;
mod env;
mod files;
mod first_run;
mod hot_corners;
mod icons;
mod inference;
mod key_row;
mod lender;
mod live;
mod marks;
mod memory;
mod page_view;
mod pictures;
mod privacy;
mod reported;
mod screen_sharing;
mod section_view;
mod sharing;
mod shortcuts;
mod sidebar;
mod spaces;
mod style_notes;
mod system;
mod ticks;
#[cfg(test)]
mod ticks_tests;
mod updates;
mod view;
mod wifi;
mod window;

pub use accounts::read_accounts;
pub use app::App;
pub use bluetooth::{BluetoothLink, BtSession, PairingReply, PairingRequest};
pub use displays::DisplaysLink;
pub use engine_install::{Desk, Quiet};
pub use env::{Env, Homes, Opening};
pub use first_run::FirstRunApp;
pub use hot_corners::{CornerDesk, NoCorners};
pub use inference::ReasonFeed;
pub use lender::Lender;
pub use live::{BusSource, Vouch};
pub use marks::ProviderMarks;
pub use pictures::{NoPictures, Pictures, ThumbFault, Thumbnails};
pub use ticks::{Gone, Ticker, Ticks};
pub use updates::{Departure, InstallRefusal, UpdateTicks, UpdatesLink};
pub use wifi::WifiLink;

/// The app's own layout stylesheet: tokens only.
pub const STYLE: &str = include_str!("style.css");
