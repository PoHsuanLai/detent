//! Following a daemon's live settings module (`org.quire.SettingsModule1`) as a pane does: look
//! for it when the pane appears, read its rows, and read again each time it comes back or says a
//! row changed. Looking never starts it (`Client::start` does that, once per page visit). Where
//! the module is looked for is `detent-system`'s `ModulesLink`; this is what a pane does with
//! what it tells.

use detent_system::bus::Daemon;
pub(crate) use detent_system::modules::Followed;
use detent_system::modules::{Module, ModulesLink};
use ds_settings::schema::KeyKind;
use std::sync::{Arc, Mutex, PoisonError};
use tokio::sync::mpsc::unbounded_channel;

/// The handle of a module while it is there.
#[derive(Debug, Clone, Default)]
pub(crate) struct Slot(Arc<Mutex<Option<Module>>>);

impl Slot {
    /// The handle now, if the module is there.
    pub(crate) fn get(&self) -> Option<Module> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The module is there with this handle, or is gone.
    pub(crate) fn put(&self, module: Option<Module>) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = module;
    }
}

/// What `daemon`'s module does, each event handled by `handle` one after the other, so a handler
/// that waits on a service holds up the next event and never the daemon's own signals. `valued`
/// picks the rows whose value is read with the rows.
pub(crate) async fn follow_with(
    modules: &dyn ModulesLink,
    daemon: &Daemon,
    valued: fn(&KeyKind) -> bool,
    mut handle: impl AsyncFnMut(Followed),
) {
    let (told, mut heard) = unbounded_channel();
    let following = modules.follow(daemon, valued, told);
    let handling = async {
        while let Some(followed) = heard.recv().await {
            handle(followed).await;
        }
    };
    tokio::join!(following, handling);
}
