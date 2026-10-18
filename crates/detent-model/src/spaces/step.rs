//! The Spaces list's one transition: `(state, input) -> (state, effects)`. Remove asks first; what
//! the sheet sends goes to memory before the registry is asked to remove the Space.

use super::removal::{RemovalSheet, SheetInput};
use crate::Absence;
use crate::Live;
use almanac_core::Removal;

/// A desktop-wide Space's id, as the registry names it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpaceKey(pub String);

/// One Space of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceRow {
    /// Its id.
    pub id: SpaceKey,
    /// What the person called it.
    pub name: String,
}

/// A list that is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpacesReady {
    /// The Spaces, in the order they were made.
    pub rows: Vec<SpaceRow>,
    /// Where a removal stands.
    pub stage: RemovalStage,
}

/// Where a removal stands: nothing, a sheet asking about one, or one under way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovalStage {
    /// No removal is asked or under way.
    Idle,
    /// The sheet asking about a removal is up.
    Asking(RemovalSheet),
    /// The Space whose removal is under way: nothing else is asked meanwhile.
    Removing(SpaceKey),
}

impl RemovalStage {
    /// The stage once the removal ended: a removal under way is over, anything else is as it was.
    fn ended(self) -> RemovalStage {
        match self {
            RemovalStage::Removing(_) => RemovalStage::Idle,
            other => other,
        }
    }
}

/// Where the list stands.
pub type SpacesState = Live<SpacesReady>;

/// What happened to the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpacesInput {
    /// The registry was read.
    Found(Result<Vec<SpaceRow>, Absence>),
    /// The person pressed Remove on a Space.
    AskRemove(SpaceKey),
    /// The person changed a choice on the sheet.
    Sheet(SheetInput),
    /// The person pressed Remove on the sheet.
    Confirm,
    /// The person pressed Cancel on the sheet.
    Cancel,
    /// The removal ended: the registry no longer lists the Space, or why it still does.
    Removed(Result<(), String>),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpacesEffect {
    /// Tell memory what becomes of the Space's memories and history, then ask the registry to
    /// remove the Space, in that order.
    Remove {
        /// The Space.
        space: SpaceKey,
        /// The person's choice.
        removal: Removal,
    },
    /// Read the registry again.
    Reload,
    /// Tell the person something went wrong.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn spaces_step(state: SpacesState, input: SpacesInput) -> (SpacesState, Vec<SpacesEffect>) {
    match input {
        SpacesInput::Found(read) => (state.found(read, fresh, refreshed), vec![]),
        other => state.inner(|ready| ready_step(ready, other)),
    }
}

/// The list once the registry is first read.
fn fresh(rows: Vec<SpaceRow>) -> SpacesReady {
    SpacesReady {
        rows,
        stage: RemovalStage::Idle,
    }
}

/// The list read again: the sheet stays while its Space is still listed.
fn refreshed(ready: SpacesReady, rows: Vec<SpaceRow>) -> SpacesReady {
    let stage = match ready.stage {
        RemovalStage::Asking(sheet) if !rows.iter().any(|row| row.id == sheet.space) => {
            RemovalStage::Idle
        }
        other => other,
    };
    SpacesReady { rows, stage }
}

fn ready_step(mut ready: SpacesReady, input: SpacesInput) -> (SpacesReady, Vec<SpacesEffect>) {
    let mut effects = Vec::new();
    match input {
        SpacesInput::AskRemove(key) if !matches!(ready.stage, RemovalStage::Removing(_)) => {
            ready.stage = ready
                .rows
                .iter()
                .find(|row| row.id == key)
                .map_or(RemovalStage::Idle, |row| {
                    RemovalStage::Asking(RemovalSheet::of(row.id.clone(), row.name.clone()))
                });
        }
        SpacesInput::Sheet(change) => {
            ready.stage = match ready.stage {
                RemovalStage::Asking(sheet) => RemovalStage::Asking(sheet.after(change)),
                other => other,
            };
        }
        SpacesInput::Confirm => {
            ready.stage = match ready.stage {
                RemovalStage::Asking(sheet) => {
                    effects.push(SpacesEffect::Remove {
                        space: sheet.space.clone(),
                        removal: sheet.removal(),
                    });
                    RemovalStage::Removing(sheet.space)
                }
                other => other,
            };
        }
        SpacesInput::Cancel => {
            ready.stage = match ready.stage {
                RemovalStage::Asking(_) => RemovalStage::Idle,
                other => other,
            };
        }
        SpacesInput::Removed(Ok(())) => {
            ready.stage = ready.stage.ended();
            effects.push(SpacesEffect::Reload);
        }
        SpacesInput::Removed(Err(why)) => {
            ready.stage = ready.stage.ended();
            effects.push(SpacesEffect::Tell(why));
            effects.push(SpacesEffect::Reload);
        }
        SpacesInput::AskRemove(_) | SpacesInput::Found(_) => {}
    }
    (ready, effects)
}
