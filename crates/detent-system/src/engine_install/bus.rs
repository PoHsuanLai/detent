//! The real desk: the engines `porter-engines-install` finds and installs, and what the assistant
//! says of its own.

use super::Desk;
use crate::bus::{BusSource, inferd};
use crate::chooser::{ChooserLink, Want, choose};
use crate::deadline::{Deadline, answered, peer};
use crate::engines::restart;
use detent_model::{EngineFailure, EngineRow, EngineStanding, EngineWay, InstallScope, Work};
use porter_dbus::{EngineState, LocalEngine};
use porter_engines::{BackendChoice, EngineId, Failure, Host, Manifest, Roots, Scope};
use porter_engines_install::{
    EngineInstalls, Fetcher, HelperProgram, InstallChoice, Parts, Standing, StartError,
};
use std::fmt;
use std::path::Path;
use std::sync::Arc;

/// The engines `porter-engines-install` finds and installs, and what the assistant says of them.
pub struct OnEngines {
    bus: BusSource,
    chooser: Arc<dyn ChooserLink>,
    installs: Option<EngineInstalls>,
}

impl fmt::Debug for OnEngines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnEngines").finish_non_exhaustive()
    }
}

impl OnEngines {
    /// The engines installed under `data_home`, the assistant looked for on `bus`, and a folder
    /// chosen through `chooser`.
    pub fn new(bus: BusSource, chooser: Arc<dyn ChooserLink>, data_home: &Path) -> Self {
        let installs = Host::read(Path::new("/"), BackendChoice::Detect).map(|host| {
            EngineInstalls::new(Parts {
                roots: Roots::standard(data_home),
                manifest: Manifest::pinned(),
                host,
                fetcher: Fetcher::system(),
                helper: HelperProgram::system(),
                strata_folder: Arc::new(|| None),
                held: porter_daemon::root_held,
            })
        });
        Self {
            bus,
            chooser,
            installs,
        }
    }

    /// The engines the assistant lists, when it is running: Settings never starts it.
    async fn reported(&self) -> Vec<LocalEngine> {
        let Ok(running) = inferd(&self.bus).await else {
            return Vec::new();
        };
        let Ok(transport) = running.inference() else {
            return Vec::new();
        };
        answered(
            Deadline::READ,
            peer::ASSISTANT,
            transport.list_local_engines(),
        )
        .await
        .and_then(Result::ok)
        .map(|rows| rows.rows)
        .unwrap_or_default()
    }

    fn row(&self, id: EngineId, reported: &[LocalEngine]) -> EngineRow {
        let listed = reported.iter().find(|engine| engine.id == id.slug());
        let here = self.installs.as_ref().map(|installs| installs.standing(id));
        let standing = match (here, listed.map(|engine| &engine.state)) {
            (Some(Standing::Installing(thousandths)), _) => EngineStanding::Installing(thousandths),
            (_, Some(EngineState::Found(_))) => EngineStanding::Ready,
            (_, Some(EngineState::Changed(_))) => EngineStanding::Changed,
            (Some(Standing::Ready(_)), _) => EngineStanding::Ready,
            (Some(Standing::Failed(why)), _) => EngineStanding::Failed(failure_of(why)),
            _ => EngineStanding::NotInstalled,
        };
        let choice = self
            .installs
            .as_ref()
            .and_then(|installs| installs.choice(id));
        EngineRow {
            id: id.slug().to_owned(),
            name: id.name().to_owned(),
            standing,
            way: way_of(id, choice),
        }
    }
}

/// Why an install ended without an engine, as the model knows it.
fn failure_of(why: Failure) -> EngineFailure {
    match why {
        Failure::NoSpace => EngineFailure::NoSpace,
        Failure::Offline => EngineFailure::Offline,
        Failure::Corrupt => EngineFailure::Corrupt,
        Failure::Declined => EngineFailure::Declined,
        Failure::Unsupported => EngineFailure::Unsupported,
        Failure::Package => EngineFailure::Package,
        Failure::Cancelled => EngineFailure::Cancelled,
        _ => EngineFailure::Other,
    }
}

/// What can be done about an engine that is not there.
fn way_of(id: EngineId, choice: Option<InstallChoice>) -> EngineWay {
    match (choice, id) {
        (Some(choice), _) => EngineWay::Download {
            scopes: choice
                .scopes
                .into_iter()
                .map(|scope| match scope {
                    Scope::AllUsers => InstallScope::AllUsers,
                    Scope::JustMe => InstallScope::JustMe,
                })
                .collect(),
            size: choice.size,
        },
        (None, EngineId::Strata) => EngineWay::Folder,
        (None, _) => EngineWay::Nothing,
    }
}

impl Desk for OnEngines {
    fn rows(&self) -> Work<'_, Vec<EngineRow>> {
        Box::pin(async move {
            let reported = self.reported().await;
            EngineId::ALL
                .into_iter()
                .map(|id| self.row(id, &reported))
                .collect()
        })
    }

    fn install(&self, engine: &str, scope: InstallScope) -> Result<(), EngineFailure> {
        let (Some(installs), Some(id)) = (&self.installs, EngineId::from_slug(engine)) else {
            return Err(EngineFailure::Other);
        };
        let scope = match scope {
            InstallScope::AllUsers => Scope::AllUsers,
            InstallScope::JustMe => Scope::JustMe,
        };
        installs.start(id, scope).map_err(|why| match why {
            StartError::Busy => EngineFailure::Busy,
            StartError::Nothing => EngineFailure::Unsupported,
            _ => EngineFailure::Other,
        })
    }

    fn cancel(&self, engine: &str) {
        if let (Some(installs), Some(id)) = (&self.installs, EngineId::from_slug(engine)) {
            installs.cancel(id);
        }
    }

    fn choose_folder(&self) -> Work<'_, Option<String>> {
        Box::pin(choose(self.chooser.as_ref(), Want::Folder))
    }

    fn point(&self, engine: &str, folder: &str) -> Work<'_, Result<(), EngineFailure>> {
        let (engine, folder) = (engine.to_owned(), folder.to_owned());
        Box::pin(async move {
            let Ok(running) = inferd(&self.bus).await else {
                return Err(EngineFailure::NotRunning);
            };
            let Ok(transport) = running.inference() else {
                return Err(EngineFailure::NotRunning);
            };
            match answered(
                Deadline::ACTION,
                peer::ASSISTANT,
                transport.point_engine(&engine, &folder),
            )
            .await
            {
                Some(Ok(())) => Ok(()),
                Some(Err(_)) | None => Err(EngineFailure::NotThere),
            }
        })
    }

    fn restart(&self) -> Work<'_, ()> {
        Box::pin(async move {
            // A restart that fails leaves the assistant as it was; the person tries Continue again.
            let _outcome = restart(&self.bus).await;
        })
    }
}
