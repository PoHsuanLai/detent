//! The real desk: the engines `porter-engines-install` finds and installs, and what the assistant
//! says of its own.

use super::desk::{Desk, Work};
use super::portal;
use crate::computers::inferd;
use crate::deadline::{Deadline, answered, peer};
use crate::engines::restart;
use crate::live::BusSource;
use detent_model::{EngineRow, EngineStanding, EngineWay, InstallScope};
use porter_dbus::{EngineState, LocalEngine};
use porter_engines::{BackendChoice, EngineId, Host, Manifest, Roots, Scope};
use porter_engines_install::{
    EngineInstalls, Fetcher, HelperProgram, InstallChoice, Parts, Standing, StartError,
};
use std::path::Path;
use std::sync::Arc;

/// The words of a download that cannot start.
const NOT_STARTED: &str = "The download could not be started.";

pub(super) struct System {
    bus: BusSource,
    installs: Option<EngineInstalls>,
}

impl System {
    pub(super) fn new(bus: BusSource, data_home: &Path) -> Self {
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
        Self { bus, installs }
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
            (Some(Standing::Failed(why)), _) => EngineStanding::Failed(why.sentence().to_owned()),
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

impl Desk for System {
    fn rows(&self) -> Work<'_, Vec<EngineRow>> {
        Box::pin(async move {
            let reported = self.reported().await;
            EngineId::ALL
                .into_iter()
                .map(|id| self.row(id, &reported))
                .collect()
        })
    }

    fn install(&self, engine: &str, scope: InstallScope) -> Result<(), String> {
        let (Some(installs), Some(id)) = (&self.installs, EngineId::from_slug(engine)) else {
            return Err(NOT_STARTED.to_owned());
        };
        let scope = match scope {
            InstallScope::AllUsers => Scope::AllUsers,
            InstallScope::JustMe => Scope::JustMe,
        };
        installs.start(id, scope).map_err(|why| match why {
            StartError::Busy => "It is already being downloaded.".to_owned(),
            StartError::Nothing => "There is no download of it for this computer.".to_owned(),
        })
    }

    fn cancel(&self, engine: &str) {
        if let (Some(installs), Some(id)) = (&self.installs, EngineId::from_slug(engine)) {
            installs.cancel(id);
        }
    }

    fn choose_folder(&self) -> Work<'_, Option<String>> {
        Box::pin(portal::choose_folder(&self.bus))
    }

    fn point(&self, engine: &str, folder: &str) -> Work<'_, Result<(), String>> {
        let (engine, folder) = (engine.to_owned(), folder.to_owned());
        Box::pin(async move {
            let name = EngineId::from_slug(&engine).map_or("it", EngineId::name);
            let refused = format!("That folder doesn't hold {name}.");
            let Ok(running) = inferd(&self.bus).await else {
                return Err("The assistant is not running.".to_owned());
            };
            let Ok(transport) = running.inference() else {
                return Err("The assistant is not running.".to_owned());
            };
            match answered(
                Deadline::ACTION,
                peer::ASSISTANT,
                transport.point_engine(&engine, &folder),
            )
            .await
            {
                Some(Ok(())) => Ok(()),
                Some(Err(_)) | None => Err(refused),
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
