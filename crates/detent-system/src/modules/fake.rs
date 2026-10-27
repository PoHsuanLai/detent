//! The modules of the daemons from memory: a [`LiveModule`] runs in this process, is handed to the
//! panes as if it were on the bus, and does what the skeleton does with each call: asks `permit`
//! for the Settings app, refuses a choice its rows list as unavailable, and, after a `set`, tells
//! whoever follows it the value a following `get` reports. A test makes a module appear, go away
//! and come back, and announces a change the module did not get through a `set`.

use super::caller::Settings;
use super::{Followed, Module, ModuleHandle, ModulesLink, reread};
use crate::bus::{Daemon, absence_of};
use crate::lock::locked;
use detent_model::{Absence, Described, Work};
use ds_settings::live::{Access, LiveError, LiveModule, LiveSchema, Verdict};
use ds_settings::schema::{KeyKind, KeyPath};
use std::fmt;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch;

/// How many changes wait for a follower that is slow to take them.
const BACKLOG: usize = 64;

/// One row's new value, as followers hear it.
type Change = (KeyPath, toml::Value);

/// Whether a daemon's module can be used now.
#[derive(Debug, Clone)]
enum Standing {
    /// It cannot, and why.
    Away(Absence),
    /// It is there.
    There(Module),
}

/// One daemon's place: where it stands and the changes it tells.
#[derive(Debug)]
struct Place {
    daemon: Daemon,
    standing: watch::Sender<Standing>,
    changes: broadcast::Sender<Change>,
}

/// The daemons' modules, each held in this process. Cheap to clone; every clone is the same set.
#[derive(Debug, Clone, Default)]
pub struct FakeModules(Arc<Mutex<Vec<Arc<Place>>>>);

impl FakeModules {
    /// No daemon is running until a module is hosted for it.
    pub fn new() -> FakeModules {
        FakeModules::default()
    }

    /// `module` is `daemon`'s now: it appears, or comes back, and every follower reads it again.
    pub fn host<M: LiveModule>(&self, daemon: &Daemon, module: M) {
        let place = self.place(daemon);
        let hosted = Hosted {
            module,
            changes: place.changes.clone(),
            settings: Settings::default(),
        };
        place
            .standing
            .send_replace(Standing::There(Module::new(Arc::new(hosted))));
    }

    /// `daemon`'s module is gone, for this reason, and every follower is told.
    pub fn away(&self, daemon: &Daemon, why: Absence) {
        self.place(daemon)
            .standing
            .send_replace(Standing::Away(why));
    }

    /// `daemon`'s module says `key` is `value` now, as it does for a change it made itself: only
    /// followers the module let read hear it.
    pub fn announce(&self, daemon: &Daemon, key: &KeyPath, value: toml::Value) {
        let _nobody_follows = self.place(daemon).changes.send((key.clone(), value));
    }

    /// The place of `daemon`, made (not running) when it has none yet.
    fn place(&self, daemon: &Daemon) -> Arc<Place> {
        let mut places = locked(&self.0);
        if let Some(place) = places.iter().find(|place| place.daemon == *daemon) {
            return Arc::clone(place);
        }
        let place = Arc::new(Place {
            daemon: daemon.clone(),
            standing: watch::channel(Standing::Away(Absence::NotRunning)).0,
            changes: broadcast::channel(BACKLOG).0,
        });
        places.push(Arc::clone(&place));
        place
    }
}

impl ModulesLink for FakeModules {
    fn follow<'a>(
        &'a self,
        daemon: &'a Daemon,
        valued: fn(&KeyKind) -> bool,
        told: UnboundedSender<Followed>,
    ) -> Work<'a, ()> {
        Box::pin(follow(self.place(daemon), valued, told))
    }
}

/// Tells `told` what the daemon at `place` is doing, as the bus's link does.
async fn follow(place: Arc<Place>, valued: fn(&KeyKind) -> bool, told: UnboundedSender<Followed>) {
    let mut standing = place.standing.subscribe();
    loop {
        let now = standing.borrow_and_update().clone();
        let moved = match now {
            Standing::Away(absence) => {
                tell(&told, Followed::Gone(absence));
                standing.changed().await.ok()
            }
            Standing::There(module) => {
                // Subscribed before the first read, so a change in between is not lost.
                let mut changes = place.changes.subscribe();
                match meet(&module, valued).await {
                    Ok(described) => {
                        tell(&told, Followed::Opened(module, described));
                        relay(&mut changes, &mut standing, &told).await
                    }
                    Err(absence) => {
                        tell(&told, Followed::Gone(absence));
                        standing.changed().await.ok()
                    }
                }
            }
        };
        if moved.is_none() {
            return;
        }
    }
}

/// The rows of `module`, once it lets the Settings app read (the bus's `changes()` finds out the
/// same way: by asking for the rows).
async fn meet(module: &Module, valued: fn(&KeyKind) -> bool) -> Result<Described, Absence> {
    module.describe().await.map_err(absence_of)?;
    reread(module, valued).await
}

/// Passes on each change until the daemon's standing changes (`Some`) or nobody holds it (`None`).
async fn relay(
    changes: &mut broadcast::Receiver<Change>,
    standing: &mut watch::Receiver<Standing>,
    told: &UnboundedSender<Followed>,
) -> Option<()> {
    loop {
        tokio::select! {
            moved = standing.changed() => return moved.ok(),
            change = changes.recv() => match change {
                Ok((path, value)) => tell(told, Followed::Changed(path, value)),
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => return standing.changed().await.ok(),
            },
        }
    }
}

/// Passes `followed` on; nobody listening any more is the pane's end, not a failure.
fn tell(told: &UnboundedSender<Followed>, followed: Followed) {
    let _gone = told.send(followed);
}

/// A module of the process, spoken to as the skeleton speaks to it.
struct Hosted<M> {
    module: M,
    changes: broadcast::Sender<Change>,
    settings: Settings,
}

impl<M> fmt::Debug for Hosted<M> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.debug_struct("Hosted").finish_non_exhaustive()
    }
}

impl<M: LiveModule> Hosted<M> {
    /// Asks the module's hook about the Settings app, and refuses as the skeleton words it.
    async fn allowed(&self, access: Access) -> Result<(), LiveError> {
        let caller = self.settings.caller().await?;
        match self.module.permit(&caller, access).await {
            Verdict::Allow => Ok(()),
            Verdict::Refuse => Err(LiveError::NotPermitted(match access {
                Access::Read => "this caller may not read the settings module".to_owned(),
                Access::Write => "this caller may not change the settings module".to_owned(),
            })),
        }
    }

    /// Refuses a `set` of a key to a choice its current rows list as unavailable.
    async fn refuse_unavailable(
        &self,
        key: &KeyPath,
        value: &toml::Value,
    ) -> Result<(), LiveError> {
        let schema = self.module.describe().await;
        let spec = schema.key.iter().find(|spec| spec.path == *key);
        match spec.map(|spec| spec.check_available(value)) {
            Some(Err(refused)) => Err(LiveError::Unavailable(refused.reason.0)),
            _ => Ok(()),
        }
    }
}

impl<M: LiveModule> ModuleHandle for Hosted<M> {
    fn describe(&self) -> Work<'_, Result<LiveSchema, LiveError>> {
        Box::pin(async move {
            self.allowed(Access::Read).await?;
            let schema = self.module.describe().await;
            schema
                .check()
                .map_err(|why| LiveError::Failed(why.to_string()))?;
            Ok(schema)
        })
    }

    fn get<'a>(&'a self, key: &'a KeyPath) -> Work<'a, Result<toml::Value, LiveError>> {
        Box::pin(async move {
            self.allowed(Access::Read).await?;
            self.module.get(key).await
        })
    }

    fn set<'a>(
        &'a self,
        key: &'a KeyPath,
        value: &'a toml::Value,
    ) -> Work<'a, Result<(), LiveError>> {
        Box::pin(async move {
            self.allowed(Access::Write).await?;
            self.refuse_unavailable(key, value).await?;
            self.module.set(key, value.clone()).await?;
            // An action row holds no value: its change carries what was sent.
            let now = self.module.get(key).await.unwrap_or_else(|_| value.clone());
            let _nobody_follows = self.changes.send((key.clone(), now));
            Ok(())
        })
    }
}
