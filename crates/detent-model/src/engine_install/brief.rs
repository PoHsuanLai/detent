//! What the offer to get an engine tells the person, as facts: which matter it is about, what can
//! be done, and how far a download has come. The window words them (`detent-ui`'s copy).

use super::offer::{EngineOffer, EngineOfferInput, EngineRef};
use super::rows::{EngineFailure, EngineWay, InstallScope};

/// What the person may do about the offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineAction {
    /// Download the engine.
    Download,
    /// Choose the folder that holds the engine.
    ChooseFolder,
    /// Do again what did not work.
    TryAgain,
    /// Leave it.
    NotNow,
    /// Acknowledge that there is nothing to get.
    Acknowledge,
    /// Stop the download.
    Cancel,
    /// The engine is there: restart the assistant.
    Continue,
}

impl EngineAction {
    /// What taking the action tells the offer.
    pub fn input(self) -> EngineOfferInput {
        match self {
            Self::Download | Self::ChooseFolder | Self::TryAgain => EngineOfferInput::Go,
            Self::NotNow | Self::Acknowledge => EngineOfferInput::NotNow,
            Self::Cancel => EngineOfferInput::Stop,
            Self::Continue => EngineOfferInput::Continue,
        }
    }
}

/// What the offer is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineMatter {
    /// The engine can be downloaded: of this size when known, again when an install was changed.
    Download {
        /// Whether an install that was changed is being replaced.
        replacing: bool,
        /// How many bytes, when that is known.
        size: Option<u64>,
    },
    /// There is nothing to download, and the person may point at a copy they already have.
    Folder,
    /// There is nothing to download for this computer.
    Unavailable,
    /// A folder the person chose is being counted.
    Checking,
    /// The engine is being downloaded.
    Downloading,
    /// The engine is there.
    Ready,
    /// It did not work, and why.
    Failed {
        /// The reason.
        reason: EngineFailure,
    },
}

/// The offer as it is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineBrief {
    /// The engine it is about.
    pub engine: EngineRef,
    /// What it is about.
    pub matter: EngineMatter,
    /// The ways to choose between; empty when there is nothing to choose.
    pub scopes: Vec<InstallScope>,
    /// The way chosen.
    pub scope: InstallScope,
    /// How far a download has come, in thousandths, when one is going on.
    pub progress: Option<u32>,
    /// Whether work with no end in sight is going on.
    pub waiting: bool,
    /// The action that goes on.
    pub primary: Option<EngineAction>,
    /// The action that leaves.
    pub secondary: Option<EngineAction>,
}

/// The facts of `offer`; `None` while nothing is offered.
pub fn engine_brief(offer: &EngineOffer) -> Option<EngineBrief> {
    let plain = |engine: &EngineRef, matter: EngineMatter| EngineBrief {
        engine: engine.clone(),
        matter,
        scopes: Vec::new(),
        scope: InstallScope::JustMe,
        progress: None,
        waiting: false,
        primary: None,
        secondary: None,
    };
    Some(match offer {
        EngineOffer::Idle => return None,
        EngineOffer::Asking {
            engine,
            way: EngineWay::Download { scopes, size },
            scope,
            replacing,
        } => EngineBrief {
            scopes: if scopes.len() > 1 {
                scopes.clone()
            } else {
                Vec::new()
            },
            scope: *scope,
            primary: Some(EngineAction::Download),
            secondary: Some(EngineAction::NotNow),
            ..plain(
                engine,
                EngineMatter::Download {
                    replacing: *replacing,
                    size: *size,
                },
            )
        },
        EngineOffer::Asking {
            engine,
            way: EngineWay::Folder,
            ..
        } => EngineBrief {
            primary: Some(EngineAction::ChooseFolder),
            secondary: Some(EngineAction::NotNow),
            ..plain(engine, EngineMatter::Folder)
        },
        EngineOffer::Asking { engine, .. } => EngineBrief {
            primary: Some(EngineAction::Acknowledge),
            ..plain(engine, EngineMatter::Unavailable)
        },
        EngineOffer::Pointing { engine } => plain(engine, EngineMatter::Checking),
        EngineOffer::Working { engine, permille } => EngineBrief {
            progress: Some(*permille),
            secondary: Some(EngineAction::Cancel),
            ..plain(engine, EngineMatter::Downloading)
        },
        EngineOffer::Done { engine } => EngineBrief {
            primary: Some(EngineAction::Continue),
            ..plain(engine, EngineMatter::Ready)
        },
        EngineOffer::Failed {
            engine,
            reason,
            scope,
            way,
        } => EngineBrief {
            scope: *scope,
            primary: match way {
                EngineWay::Nothing => None,
                EngineWay::Download { .. } | EngineWay::Folder => Some(EngineAction::TryAgain),
            },
            secondary: Some(EngineAction::NotNow),
            ..plain(engine, EngineMatter::Failed { reason: *reason })
        },
    })
}
