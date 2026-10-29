//! The real desk: the engines stoker's `engine-install` finds and installs, and what the assistant
//! says of its own.

use super::Desk;
use crate::bus::{BusSource, inferd};
use crate::chooser::{ChooserLink, Want, choose};
use crate::deadline::{Deadline, answered, peer};
use crate::engines::restart;
use detent_model::{EngineFailure, EngineRow, EngineStanding, EngineWay, InstallScope, Work};
use engine_install::{
    EngineInstalls, Fetcher, HelperProgram, InstallChoice, Parts, Standing, StartError,
};
use engine_manifest::{BackendChoice, Failure, Manifest, engine_from_slug};
use engine_probe::{Roots, Scope, read_host};
use model_catalog::EngineKind;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use steward_dbus::inference::{EngineState, LocalEngine};

/// The engines Settings offers, in the order the page lists them: the language-model engines. The
/// speech engines are not downloaded from here.
const SHOWN: [EngineKind; 3] = [
    EngineKind::LlamaServer,
    EngineKind::Vllm,
    EngineKind::Strata,
];

/// Polkit's launcher, and the install helper stoker's install step places.
const PKEXEC: &str = "/usr/bin/pkexec";
const HELPER: &str = "/usr/libexec/quire/engine-helper";

/// What the person calls the engine (copy text: the engine spec holds no words a person reads).
fn name_of(kind: EngineKind) -> &'static str {
    match kind {
        EngineKind::LlamaServer => "llama.cpp",
        EngineKind::Vllm => "vLLM",
        EngineKind::Strata => "Strata",
        EngineKind::SpeechHost => "the speech engine",
        EngineKind::KokoroFastApi => "Kokoro",
    }
}

/// The engines stoker's `engine-install` finds and installs, and what the assistant says of them.
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
        let installs = read_host(Path::new("/"), BackendChoice::Detect).map(|host| {
            EngineInstalls::new(Parts {
                roots: Roots::standard(data_home),
                manifest: Manifest::pinned(),
                host,
                fetcher: Fetcher::system(),
                helper: HelperProgram {
                    pkexec: PathBuf::from(PKEXEC),
                    program: PathBuf::from(HELPER),
                },
                strata_folder: Arc::new(|| None),
                held: porter_callers::root_held,
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

    fn row(&self, kind: EngineKind, reported: &[LocalEngine]) -> EngineRow {
        let word = engine_spec::engine(kind);
        let listed = reported.iter().find(|engine| engine.id == word.bus_word());
        let here = self
            .installs
            .as_ref()
            .map(|installs| installs.standing(kind));
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
            .and_then(|installs| installs.choice(kind));
        EngineRow {
            id: word.disk_word().to_owned(),
            name: name_of(kind).to_owned(),
            standing,
            way: way_of(kind, choice),
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
fn way_of(kind: EngineKind, choice: Option<InstallChoice>) -> EngineWay {
    match (choice, kind) {
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
        (None, EngineKind::Strata) => EngineWay::Folder,
        (None, _) => EngineWay::Nothing,
    }
}

impl Desk for OnEngines {
    fn rows(&self) -> Work<'_, Vec<EngineRow>> {
        Box::pin(async move {
            let reported = self.reported().await;
            SHOWN
                .into_iter()
                .map(|kind| self.row(kind, &reported))
                .collect()
        })
    }

    fn install(&self, engine: &str, scope: InstallScope) -> Result<(), EngineFailure> {
        let (Some(installs), Some(kind)) = (&self.installs, engine_from_slug(engine)) else {
            return Err(EngineFailure::Other);
        };
        let scope = match scope {
            InstallScope::AllUsers => Scope::AllUsers,
            InstallScope::JustMe => Scope::JustMe,
        };
        installs.start(kind, scope).map_err(|why| match why {
            StartError::Busy => EngineFailure::Busy,
            StartError::Nothing => EngineFailure::Unsupported,
            _ => EngineFailure::Other,
        })
    }

    fn cancel(&self, engine: &str) {
        if let (Some(installs), Some(kind)) = (&self.installs, engine_from_slug(engine)) {
            installs.cancel(kind);
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
