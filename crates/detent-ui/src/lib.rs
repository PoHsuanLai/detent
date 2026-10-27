//! The Settings window: a search field and the page list in a sidebar, the selected page on the
//! right, drawn only with quire's components. Values are read and written through
//! `detent-files`; the programs' own watches apply them.

mod accounts;
mod advanced_view;
mod app;
mod assistant_model;
mod assistants;
mod bluetooth;
mod computers;
mod controls;
mod copy;
mod displays;
mod engine_install;
mod engines;
mod env;
mod extensions;
mod files;
mod first_run;
mod hot_corners;
mod icons;
mod key_row;
mod live;
mod marks;
mod memory_consent;
mod page_view;
mod picker;
mod privacy;
mod reported;
mod running;
mod screen_sharing;
mod section_view;
mod sharing;
mod shortcuts;
mod sidebar;
mod spaces;
mod style_notes;
mod system;
mod updates;
mod view;
mod wifi;
mod window;

pub use app::App;
pub use detent_system::accounts::read_accounts;
pub use detent_system::bluetooth::{BluetoothLink, BtSession, PairingReply, PairingRequest};
pub use detent_system::bus::{BusSource, Vouch};
pub use detent_system::displays::DisplaysLink;
pub use detent_system::engine_install::{Desk, Quiet};
pub use detent_system::hot_corners::{CornerDesk, NoCorners};
pub use detent_system::lending::Lender;
pub use detent_system::updates::{Departure, InstallRefusal, UpdateTicks, UpdatesLink};
pub use detent_system::wallpaper::{NoPictures, Pictures, ThumbFault, Thumbnails};
pub use detent_system::wifi::WifiLink;
pub use detent_system::{Gone, Ticker, Ticks};
pub use env::{Env, Homes, Opening};
pub use first_run::FirstRunApp;
pub use marks::ProviderMarks;
pub use picker::ReasonFeed;

/// The app's own layout stylesheet: tokens only.
pub const STYLE: &str = include_str!("style.css");
