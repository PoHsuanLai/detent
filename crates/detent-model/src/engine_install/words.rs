//! What the sheet says: Mac wording, and no path, program name or protocol.

use super::rows::{EngineWay, InstallScope};
use super::sheet::{EngineRef, EngineSheet, EngineSheetInput, Replacing};

/// What a button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// Go on: download, choose a folder, try again.
    Go,
    /// Leave it.
    NotNow,
    /// Stop the download.
    Stop,
    /// The engine is there: restart the assistant.
    Continue,
}

impl Press {
    /// The input a press makes.
    pub fn input(self) -> EngineSheetInput {
        match self {
            Self::Go => EngineSheetInput::Go,
            Self::NotNow => EngineSheetInput::NotNow,
            Self::Stop => EngineSheetInput::Stop,
            Self::Continue => EngineSheetInput::Continue,
        }
    }
}

/// A button: its words and what it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SheetButton {
    /// What it says.
    pub label: &'static str,
    /// What it does.
    pub press: Press,
}

/// The sheet as it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetWords {
    /// The heading.
    pub title: String,
    /// The sentence under it.
    pub message: String,
    /// The ways to choose between ("Install for"); empty when there is nothing to choose.
    pub scopes: Vec<InstallScope>,
    /// The way chosen.
    pub scope: InstallScope,
    /// How far a download has come, when one is going on.
    pub progress: Option<u32>,
    /// Whether work with no end in sight is going on (a folder being checked).
    pub waiting: Waiting,
    /// The button that goes on, bottom right.
    pub primary: Option<SheetButton>,
    /// The button that leaves.
    pub secondary: Option<SheetButton>,
}

/// Whether the sheet shows a bar that only says something is going on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waiting {
    /// No.
    No,
    /// Yes.
    Yes,
}

/// What the label above the choice of who it is for says.
pub const INSTALL_FOR: &str = "Install for";

fn button(label: &'static str, press: Press) -> Option<SheetButton> {
    Some(SheetButton { label, press })
}

/// A size as a person says it: `30 MB`, `4.4 GB` (decimal units, as the Mac does).
pub fn size_words(bytes: u64) -> String {
    const MB: u64 = 1_000_000;
    const GB: u64 = 1_000_000_000;
    if bytes >= GB - 5 * MB {
        // One decimal place, by whole tenths so no float is needed.
        let tenths = (bytes + GB / 20) / (GB / 10);
        format!("{}.{} GB", tenths / 10, tenths % 10)
    } else {
        format!("{} MB", ((bytes + MB / 2) / MB).max(1))
    }
}

fn download_message(engine: &EngineRef, way: &EngineWay, replacing: Replacing) -> String {
    let name = &engine.name;
    let lead = match replacing {
        Replacing::No => format!("Your assistant needs {name} to run models on this computer."),
        Replacing::Yes => format!(
            "{name} was changed since it was installed, so your assistant won't use it. \
             Download it again to run models on this computer."
        ),
    };
    match way {
        EngineWay::Download {
            size: Some(bytes), ..
        } => format!("{lead} It's about {}.", size_words(*bytes)),
        _ => lead,
    }
}

/// The words of `sheet`; `None` while there is no sheet.
pub fn sheet_words(sheet: &EngineSheet) -> Option<SheetWords> {
    let plain = |title: String, message: String| SheetWords {
        title,
        message,
        scopes: Vec::new(),
        scope: InstallScope::JustMe,
        progress: None,
        waiting: Waiting::No,
        primary: None,
        secondary: None,
    };
    Some(match sheet {
        EngineSheet::Closed => return None,
        EngineSheet::Asking {
            engine,
            way: way @ EngineWay::Download { scopes, .. },
            scope,
            replacing,
        } => SheetWords {
            scopes: if scopes.len() > 1 {
                scopes.clone()
            } else {
                Vec::new()
            },
            scope: *scope,
            primary: button("Download", Press::Go),
            secondary: button("Not Now", Press::NotNow),
            ..plain(
                match replacing {
                    Replacing::No => format!("Download {}?", engine.name),
                    Replacing::Yes => format!("Download {} Again?", engine.name),
                },
                download_message(engine, way, *replacing),
            )
        },
        EngineSheet::Asking {
            engine,
            way: EngineWay::Folder,
            ..
        } => SheetWords {
            primary: button("Choose Folder\u{2026}", Press::Go),
            secondary: button("Not Now", Press::NotNow),
            ..plain(
                format!("{} Isn't Available to Download", engine.name),
                format!(
                    "There is no ready-made download of {} for this computer. If you already \
                     have it, choose its folder.",
                    engine.name
                ),
            )
        },
        EngineSheet::Asking { engine, .. } => SheetWords {
            primary: button("OK", Press::NotNow),
            ..plain(
                format!("{} Isn't Available", engine.name),
                format!("There is no download of {} for this computer.", engine.name),
            )
        },
        EngineSheet::Pointing { engine } => {
            plain(format!("Checking {}\u{2026}", engine.name), String::new())
        }
        EngineSheet::Working { engine, permille } => SheetWords {
            progress: Some(*permille),
            secondary: button("Cancel", Press::Stop),
            ..plain(
                format!("Downloading {}\u{2026}", engine.name),
                String::new(),
            )
        },
        EngineSheet::Done { engine } => SheetWords {
            primary: button("Continue", Press::Continue),
            ..plain(
                format!("{} Is Ready", engine.name),
                "Your assistant restarts to use it.".to_owned(),
            )
        },
        EngineSheet::Failed {
            engine,
            reason,
            scope,
            way,
        } => SheetWords {
            scope: *scope,
            primary: match way {
                EngineWay::Nothing => None,
                EngineWay::Download { .. } | EngineWay::Folder => button("Try Again", Press::Go),
            },
            secondary: button("Not Now", Press::NotNow),
            ..plain(format!("Couldn't Download {}", engine.name), reason.clone())
        },
    })
}
