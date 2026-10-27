//! The one door the Settings window uses to reach the computer. A [`Client`] holds every service
//! (the links of `detent-system`) and the bus they are looked for on; the window asks it for a
//! pane and gets a live value to draw and the commands to give it. Every follow loop, deadline and
//! error mapping lives here, so a view holds none: it reads a [`Live`] state, draws it, and sends
//! the pane an input.
//!
//! The state of a pane is its model's (`detent_model::<pane>::State`); the client applies the
//! model's `step` and tells the window the new state through the pane's [`Events`]. What the person
//! is told comes the same way, as a [`Notice`] that names a fact, never a sentence. Nothing here
//! draws, and nothing here names a UI crate.

mod accounts;
mod bluetooth;
mod builder;
mod chooser;
mod client;
mod computers;
mod displays;
mod engine_install;
mod engines;
mod files;
mod follow;
mod hot_corners;
mod lending;
mod live;
#[cfg(test)]
mod live_tests;
mod marks;
mod memory_consent;
mod notice;
mod pane;
#[cfg(test)]
mod pane_tests;
mod picker;
mod privacy;
mod running;
mod screen_sharing;
mod spaces;
mod updates;
mod wait;
mod wallpaper;
mod whoami;
mod wifi;

pub use accounts::{Accounts, Datasets, HeldDeletions, Listed, Listing};
pub use bluetooth::Bluetooth;
pub use builder::{ClientBuilder, ClientHomes};
pub use chooser::{Asked, Chooser, Want};
pub use client::Client;
pub use computers::Computers;
pub use displays::Displays;
pub use engine_install::EngineInstall;
pub use engines::{EnginePrograms, Engines};
pub use files::{
    AgentsTool, ConfigHome, DataHome, ExtensionHome, FileValues, FilesError, FirstRunMark,
    KeymapFiles, StateHome, StyleInputs, add_agent, installed_agents, load, load_agents,
    load_places, remove_agent, save, save_agent_choice, save_places,
};
pub use hot_corners::Corners;
pub use lending::Lending;
pub use live::{Events, Heard, Live};
pub use marks::{MarkFacts, ProviderMarkFacts, shipped_marks};
pub use memory_consent::MemoryConsent;
pub use notice::Notice;
pub use pane::{Pane, Pending};
pub use picker::{Models, Picker, ReasonFeed, Reasons};
pub use privacy::Privacy;
pub use running::Running;
pub use screen_sharing::ScreenSharing;
pub use spaces::Spaces;
pub use updates::Updates;
pub use wait::{Deadline, Gone, Ticker, Ticks, answered, peer};
pub use wallpaper::Wallpaper;
pub use whoami::Whoami;
pub use wifi::Wifi;

// The services the tests and the binary hand to a builder, and the types their signatures name.
pub use detent_system::bluetooth::{BluetoothLink, BtSession, PairingReply, PairingRequest};
pub use detent_system::bus::{BusSource, Vouch};
pub use detent_system::displays::DisplaysLink;
pub use detent_system::engine_install::{Desk, Quiet};
pub use detent_system::hot_corners::{CornerDesk, NoCorners};
pub use detent_system::lending::Lender;
pub use detent_system::person::{NoPerson, PersonLink};
pub use detent_system::updates::{Departure, InstallRefusal, UpdateTicks, UpdatesLink};
pub use detent_system::wallpaper::{NoPictures, Pictures, ThumbFault, Thumbnails};
pub use detent_system::wifi::WifiLink;
