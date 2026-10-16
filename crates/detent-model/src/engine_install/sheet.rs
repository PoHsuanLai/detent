//! The sheet: asked, working, done or failed, and what each press does.

use super::rows::{EngineRow, EngineStanding, EngineWay, InstallScope};

/// Which engine the sheet is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineRef {
    /// Its id on the assistant's bus.
    pub id: String,
    /// What the person calls it.
    pub name: String,
}

/// What the sheet shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EngineSheet {
    /// No sheet.
    #[default]
    Closed,
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
    /// The engine is there; "Continue" restarts the assistant so it finds it.
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
        /// The plain reason.
        reason: String,
    },
}

/// Whether the sheet is about an engine whose install was changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replacing {
    /// The engine is not there.
    No,
    /// An install of it is there and was changed since it was made.
    Yes,
}

/// What the sheet knows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EngineSheetState {
    /// The engines as the assistant lists them now.
    pub rows: Vec<EngineRow>,
    /// The sheet.
    pub sheet: EngineSheet,
}

/// What happens to the sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineSheetInput {
    /// The engines were looked at again.
    Rows(Vec<EngineRow>),
    /// The person chose the engine with this id in "Runs with".
    Chose(String),
    /// The person picked who it is for.
    Scope(InstallScope),
    /// "Download" (or "Choose Folder…", or "Try Again") was pressed.
    Go,
    /// "Not Now", or "OK", was pressed.
    NotNow,
    /// "Cancel" was pressed on a download.
    Stop,
    /// "Continue" was pressed on a download that is done.
    Continue,
    /// A folder was chosen (`None`: the chooser was closed).
    Folder(Option<String>),
    /// The assistant was asked to count that folder; `Err` carries the reason.
    Pointed(Result<(), String>),
    /// The install could not be started; carries the reason.
    NotStarted(String),
}

/// What the window does next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineSheetEffect {
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

fn asking(row: &EngineRow) -> EngineSheet {
    EngineSheet::Asking {
        engine: reference(row),
        way: row.way.clone(),
        scope: first_scope(&row.way),
        replacing: match row.standing {
            EngineStanding::Changed => Replacing::Yes,
            _ => Replacing::No,
        },
    }
}

/// The sheet after the engines were looked at again: a download in progress follows its row.
fn followed(sheet: EngineSheet, rows: &[EngineRow]) -> EngineSheet {
    let EngineSheet::Working { engine, .. } = sheet else {
        return sheet;
    };
    let Some(row) = rows.iter().find(|row| row.id == engine.id) else {
        return EngineSheet::Closed;
    };
    match &row.standing {
        EngineStanding::Installing(now) => EngineSheet::Working {
            engine,
            permille: *now,
        },
        EngineStanding::Ready => EngineSheet::Done { engine },
        EngineStanding::Failed(reason) => EngineSheet::Failed {
            engine,
            way: row.way.clone(),
            scope: first_scope(&row.way),
            reason: reason.clone(),
        },
        // Stopped: nothing was installed, and nothing is said.
        EngineStanding::NotInstalled | EngineStanding::Changed => EngineSheet::Closed,
    }
}

/// The next state, and what to do about it.
pub fn engine_sheet_step(
    state: EngineSheetState,
    input: EngineSheetInput,
) -> (EngineSheetState, Vec<EngineSheetEffect>) {
    let EngineSheetState { rows, sheet } = state;
    let (sheet, rows, effects) = match (sheet, input) {
        (sheet, EngineSheetInput::Rows(fresh)) => (followed(sheet, &fresh), fresh, vec![]),
        (EngineSheet::Closed, EngineSheetInput::Chose(id)) => {
            let sheet = match rows.iter().find(|row| row.id == id) {
                Some(row) if row.needs_getting() => match row.standing {
                    EngineStanding::Installing(permille) => EngineSheet::Working {
                        engine: reference(row),
                        permille,
                    },
                    _ => asking(row),
                },
                _ => EngineSheet::Closed,
            };
            (sheet, rows, vec![])
        }
        (
            EngineSheet::Asking {
                engine,
                way,
                scope: _,
                replacing,
            },
            EngineSheetInput::Scope(scope),
        ) => (
            EngineSheet::Asking {
                engine,
                way,
                scope,
                replacing,
            },
            rows,
            vec![],
        ),
        (
            EngineSheet::Asking {
                engine,
                way: EngineWay::Download { .. },
                scope,
                ..
            },
            EngineSheetInput::Go,
        ) => {
            let effect = EngineSheetEffect::Install {
                engine: engine.id.clone(),
                scope,
            };
            (
                EngineSheet::Working {
                    engine,
                    permille: 0,
                },
                rows,
                vec![effect],
            )
        }
        (
            sheet @ EngineSheet::Asking {
                way: EngineWay::Folder,
                ..
            },
            EngineSheetInput::Go,
        ) => (sheet, rows, vec![EngineSheetEffect::ChooseFolder]),
        (sheet @ EngineSheet::Asking { .. }, EngineSheetInput::Go) => (sheet, rows, vec![]),
        (EngineSheet::Asking { engine, .. }, EngineSheetInput::Folder(Some(folder))) => (
            EngineSheet::Pointing {
                engine: engine.clone(),
            },
            rows,
            vec![EngineSheetEffect::Point {
                engine: engine.id,
                folder,
            }],
        ),
        (sheet @ EngineSheet::Asking { .. }, EngineSheetInput::Folder(None)) => {
            (sheet, rows, vec![])
        }
        (EngineSheet::Pointing { engine }, EngineSheetInput::Pointed(Ok(()))) => {
            (EngineSheet::Done { engine }, rows, vec![])
        }
        (EngineSheet::Pointing { engine }, EngineSheetInput::Pointed(Err(reason)))
        | (EngineSheet::Working { engine, .. }, EngineSheetInput::NotStarted(reason)) => {
            let way = rows
                .iter()
                .find(|row| row.id == engine.id)
                .map_or(EngineWay::Nothing, |row| row.way.clone());
            let scope = first_scope(&way);
            (
                EngineSheet::Failed {
                    engine,
                    way,
                    scope,
                    reason,
                },
                rows,
                vec![],
            )
        }
        (sheet @ EngineSheet::Working { .. }, EngineSheetInput::Stop) => {
            let effect = match &sheet {
                EngineSheet::Working { engine, .. } => {
                    vec![EngineSheetEffect::Cancel(engine.id.clone())]
                }
                _ => vec![],
            };
            (sheet, rows, effect)
        }
        (EngineSheet::Done { .. }, EngineSheetInput::Continue) => {
            (EngineSheet::Closed, rows, vec![EngineSheetEffect::Restart])
        }
        (
            EngineSheet::Failed {
                engine,
                way,
                scope,
                reason: _,
            },
            EngineSheetInput::Go,
        ) => {
            let replacing =
                rows.iter()
                    .find(|row| row.id == engine.id)
                    .map_or(Replacing::No, |row| match row.standing {
                        EngineStanding::Changed => Replacing::Yes,
                        _ => Replacing::No,
                    });
            (
                EngineSheet::Asking {
                    engine,
                    way,
                    scope,
                    replacing,
                },
                rows,
                vec![],
            )
        }
        (EngineSheet::Asking { .. } | EngineSheet::Failed { .. }, EngineSheetInput::NotNow) => {
            (EngineSheet::Closed, rows, vec![])
        }
        (sheet, _) => (sheet, rows, vec![]),
    };
    (EngineSheetState { rows, sheet }, effects)
}
