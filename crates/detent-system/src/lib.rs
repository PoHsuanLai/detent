//! Every service Settings reaches: the bus, the portal, NetworkManager, BlueZ, the compositor,
//! accountd, inferd, docket, memory and syncd. One module per service, named as its model is, each
//! with a `Link` trait that takes and returns `detent-model` facts, the real implementation (zbus
//! proxies, wire decoding, error to `Absence`), and an in-memory fake under the `testing` feature.
//! Pure rules found while reading the wire live in `detent-model`; nothing here draws.

pub mod accounts;
pub mod bluetooth;
pub mod bus;
pub mod chooser;
pub mod computers;
mod deadline;
#[cfg(test)]
mod deadline_tests;
pub mod displays;
pub mod engine_install;
pub mod engines;
#[cfg(feature = "testing")]
mod fake_kit;
pub mod hot_corners;
pub mod lending;
#[cfg(feature = "testing")]
mod lock;
pub mod memory_consent;
pub mod picker;
pub mod privacy;
pub mod running;
pub mod screen_sharing;
pub mod spaces;
pub mod sync;
mod ticks;
#[cfg(test)]
mod ticks_tests;
pub mod updates;
pub mod wallpaper;
pub mod wifi;

pub use deadline::{Deadline, answered, peer};
pub use ticks::{Gone, Ticker, Ticks};
