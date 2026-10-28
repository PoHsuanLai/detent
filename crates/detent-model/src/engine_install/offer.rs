//! The offer to get an engine: asked, working, done or failed, and what each answer does.

use super::rows::{EngineFailure, EngineRow, EngineStanding, EngineWay, InstallScope};

/// Which engine the offer is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineRef {
    /// Its id on the assistant's bus.
    pub id: String,
    /// What the person calls it.
    pub name: String,
}

/// Where the offer to get an engine stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EngineOffer {
    /// Nothing is offered.
    #[default]
    Idle,
    /// It asks whether to get the engine.
    Asking {
        /// The engine.
        engine: EngineRef,
        /// What can be done.
        way: EngineWay,
        /// For whom, when it is downloaded.
        scope: InstallScope,
        /// Whether an install that was changed is being replaced.
        replacing: Replacing,
    },
    /// A folder the person chose is being counted as the engine's.
    Pointing {
        /// The engine.
        engine: EngineRef,
    },
    /// The engine is being downloaded.
    Working {
        /// The engine.
        engine: EngineRef,
        /// Thousandths done.
        permille: u32,
    },
    /// The engine is there; going on restarts the assistant so it finds it.
    Done {
        /// The engine.
        engine: EngineRef,
    },
    /// It did not work, and why.
    Failed {
        /// The engine.
        engine: EngineRef,
        /// What can be done.
        way: EngineWay,
        /// For whom it was tried.
        scope: InstallScope,
        /// The reason.
        reason: EngineFailure,
    },
}

/// Whether the offer is about an engine whose install was changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replacing {
    /// The engine is not there.
    No,
    /// An install of it is there and was changed since it was made.
    Yes,
}

/// What the offer knows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EngineOfferState {
    /// The engines as the assistant lists them now.
    pub rows: Vec<EngineRow>,
    /// Where the offer stands.
    pub offer: EngineOffer,
}

/// What happens to the offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineOfferInput {
    /// The engines were looked at again.
    Rows(Vec<EngineRow>),
    /// The person chose the engine with this id in "Runs with".
    Chose(String),
    /// The person picked who it is for.
    Scope(InstallScope),
    /// The person said go on: download, choose a folder, or try again.
    Go,
    /// The person left it: not now, or acknowledged that there is nothing to get.
    NotNow,
    /// The person stopped a download.
    Stop,
    /// The person went on after a download that is done.
    Continue,
    /// A folder was chosen (`None`: the chooser was closed).
    Folder(Option<String>),
    /// The assistant was asked to count that folder; `Err` carries the reason.
    Pointed(Result<(), EngineFailure>),
    /// The install could not be started; carries the reason.
    NotStarted(EngineFailure),
}

/// What the window does next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineOfferEffect {
    /// Start downloading `engine` for `scope`.
    Install {
        /// The engine's id.
        engine: String,
        /// For whom.
        scope: InstallScope,
    },
    /// Stop the download of `engine`.
    Cancel(String),
    /// Open the folder chooser.
    ChooseFolder,
    /// Count `folder` as `engine`'s.
    Point {
        /// The engine's id.
        engine: String,
        /// The folder.
        folder: String,
    },
    /// Restart the assistant so it finds what was installed.
    Restart,
}

fn reference(row: &EngineRow) -> EngineRef {
    EngineRef {
        id: row.id.clone(),
        name: row.name.clone(),
    }
}

/// The scope to start from: everyone's when it is offered, else the person's own.
fn first_scope(way: &EngineWay) -> InstallScope {
    match way {
        EngineWay::Download { scopes, .. } => {
            scopes.first().copied().unwrap_or(InstallScope::JustMe)
        }
        EngineWay::Folder | EngineWay::Nothing => InstallScope::JustMe,
    }
}

fn asking(row: &EngineRow) -> EngineOffer {
    EngineOffer::Asking {
        engine: reference(row),
        way: row.way.clone(),
        scope: first_scope(&row.way),
        replacing: match row.standing {
            EngineStanding::Changed => Replacing::Yes,
            _ => Replacing::No,
        },
    }
}

/// The offer after the engines were looked at again: a download in progress follows its row.
fn followed(offer: EngineOffer, rows: &[EngineRow]) -> EngineOffer {
    let EngineOffer::Working { engine, .. } = offer else {
        return offer;
    };
    let Some(row) = rows.iter().find(|row| row.id == engine.id) else {
        return EngineOffer::Idle;
    };
    match &row.standing {
        EngineStanding::Installing(now) => EngineOffer::Working {
            engine,
            permille: *now,
        },
        EngineStanding::Ready => EngineOffer::Done { engine },
        EngineStanding::Failed(reason) => EngineOffer::Failed {
            engine,
            way: row.way.clone(),
            scope: first_scope(&row.way),
            reason: *reason,
        },
        // Stopped: nothing was installed, and nothing is said.
        EngineStanding::NotInstalled | EngineStanding::Changed => EngineOffer::Idle,
    }
}

/// The next state, and what to do about it.
pub fn engine_offer_step(
    state: EngineOfferState,
    input: EngineOfferInput,
) -> (EngineOfferState, Vec<EngineOfferEffect>) {
    let EngineOfferState { rows, offer } = state;
    let (offer, rows, effects) = match (offer, input) {
        (offer, EngineOfferInput::Rows(fresh)) => (followed(offer, &fresh), fresh, vec![]),
        (EngineOffer::Idle, EngineOfferInput::Chose(id)) => {
            let offer = match rows.iter().find(|row| row.id == id) {
                Some(row) if row.needs_getting() => match row.standing {
                    EngineStanding::Installing(permille) => EngineOffer::Working {
                        engine: reference(row),
                        permille,
                    },
                    _ => asking(row),
                },
                _ => EngineOffer::Idle,
            };
            (offer, rows, vec![])
        }
        (
            EngineOffer::Asking {
                engine,
                way,
                scope: _,
                replacing,
            },
            EngineOfferInput::Scope(scope),
        ) => (
            EngineOffer::Asking {
                engine,
                way,
                scope,
                replacing,
            },
            rows,
            vec![],
        ),
        (
            EngineOffer::Asking {
                engine,
                way: EngineWay::Download { .. },
                scope,
                ..
            },
            EngineOfferInput::Go,
        ) => {
            let effect = EngineOfferEffect::Install {
                engine: engine.id.clone(),
                scope,
            };
            (
                EngineOffer::Working {
                    engine,
                    permille: 0,
                },
                rows,
                vec![effect],
            )
        }
        (
            offer @ EngineOffer::Asking {
                way: EngineWay::Folder,
                ..
            },
            EngineOfferInput::Go,
        ) => (offer, rows, vec![EngineOfferEffect::ChooseFolder]),
        (offer @ EngineOffer::Asking { .. }, EngineOfferInput::Go) => (offer, rows, vec![]),
        (EngineOffer::Asking { engine, .. }, EngineOfferInput::Folder(Some(folder))) => (
            EngineOffer::Pointing {
                engine: engine.clone(),
            },
            rows,
            vec![EngineOfferEffect::Point {
                engine: engine.id,
                folder,
            }],
        ),
        (offer @ EngineOffer::Asking { .. }, EngineOfferInput::Folder(None)) => {
            (offer, rows, vec![])
        }
        (EngineOffer::Pointing { engine }, EngineOfferInput::Pointed(Ok(()))) => {
            (EngineOffer::Done { engine }, rows, vec![])
        }
        (EngineOffer::Pointing { engine }, EngineOfferInput::Pointed(Err(reason)))
        | (EngineOffer::Working { engine, .. }, EngineOfferInput::NotStarted(reason)) => {
            let way = rows
                .iter()
                .find(|row| row.id == engine.id)
                .map_or(EngineWay::Nothing, |row| row.way.clone());
            let scope = first_scope(&way);
            (
                EngineOffer::Failed {
                    engine,
                    way,
                    scope,
                    reason,
                },
                rows,
                vec![],
            )
        }
        (offer @ EngineOffer::Working { .. }, EngineOfferInput::Stop) => {
            let effect = match &offer {
                EngineOffer::Working { engine, .. } => {
                    vec![EngineOfferEffect::Cancel(engine.id.clone())]
                }
                _ => vec![],
            };
            (offer, rows, effect)
        }
        (EngineOffer::Done { .. }, EngineOfferInput::Continue) => {
            (EngineOffer::Idle, rows, vec![EngineOfferEffect::Restart])
        }
        (
            EngineOffer::Failed {
                engine,
                way,
                scope,
                reason: _,
            },
            EngineOfferInput::Go,
        ) => {
            let replacing =
                rows.iter()
                    .find(|row| row.id == engine.id)
                    .map_or(Replacing::No, |row| match row.standing {
                        EngineStanding::Changed => Replacing::Yes,
                        _ => Replacing::No,
                    });
            (
                EngineOffer::Asking {
                    engine,
                    way,
                    scope,
                    replacing,
                },
                rows,
                vec![],
            )
        }
        (EngineOffer::Asking { .. } | EngineOffer::Failed { .. }, EngineOfferInput::NotNow) => {
            (EngineOffer::Idle, rows, vec![])
        }
        (offer, _) => (offer, rows, vec![]),
    };
    (EngineOfferState { rows, offer }, effects)
}
