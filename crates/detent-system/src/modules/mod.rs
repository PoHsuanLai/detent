//! Following a daemon's live settings module (`org.quire.SettingsModule1`) as a pane does: look
//! for it when the pane appears, read its rows, and read again each time it comes back or says a
//! row changed. Looking never starts it (`Client::start` does that, once per page visit).

mod bus;
#[cfg(feature = "testing")]
mod caller;
#[cfg(feature = "testing")]
mod fake;
mod handle;

pub use bus::OnBus;
#[cfg(feature = "testing")]
pub use fake::FakeModules;
pub use handle::{Module, ModuleHandle, reread};

use crate::bus::Daemon;
use detent_model::{Absence, Described, Work};
use ds_settings::schema::{KeyKind, KeyPath};
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;

/// What is learned about the daemon, in the order it happens.
#[derive(Debug)]
pub enum Followed {
    /// The daemon is there: its handle and its rows.
    Opened(Module, Described),
    /// The daemon cannot be used, and why.
    Gone(Absence),
    /// The daemon put a row's value in force.
    Changed(KeyPath, toml::Value),
}

/// Where a pane looks for a daemon's live module.
pub trait ModulesLink: Debug + Send + Sync {
    /// Tells `told` what `daemon`'s module is doing until the bus goes away (or, for a fake, until
    /// nobody holds it); `valued` picks the rows whose value is read with the rows. A daemon that
    /// is not running is reported at once and looked for again when it appears; one that stops is
    /// reported gone.
    fn follow<'a>(
        &'a self,
        daemon: &'a Daemon,
        valued: fn(&KeyKind) -> bool,
        told: UnboundedSender<Followed>,
    ) -> Work<'a, ()>;
}
