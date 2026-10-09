//! The Spaces list's one transition: `(state, input) -> (state, effects)`. Remove asks first; what
//! the sheet sends goes to memory before the registry is asked to remove the Space.

use super::removal::{RemovalSheet, SheetInput};
use crate::picker::Absence;
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
    /// The sheet asking about a removal, if one is up.
    pub sheet: Option<RemovalSheet>,
    /// The Space whose removal is under way: nothing else is asked meanwhile.
    pub removing: Option<SpaceKey>,
}

/// Where the list stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SpacesState {
    /// Looking for the registry.
    #[default]
    Looking,
    /// The registry cannot be used, and why.
    Unavailable(Absence),
    /// The registry answered.
    Ready(SpacesReady),
}

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
    match (state, input) {
        (SpacesState::Ready(ready), SpacesInput::Found(Ok(rows))) => {
            let sheet = ready
                .sheet
                .filter(|sheet| rows.iter().any(|row| row.id == sheet.space));
            let state = SpacesReady {
                rows,
                sheet,
                ..ready
            };
            (SpacesState::Ready(state), vec![])
        }
        (_, SpacesInput::Found(Ok(rows))) => (
            SpacesState::Ready(SpacesReady {
                rows,
                sheet: None,
                removing: None,
            }),
            vec![],
        ),
        (_, SpacesInput::Found(Err(absence))) => (SpacesState::Unavailable(absence), vec![]),
        (SpacesState::Ready(ready), input) => {
            let (ready, effects) = ready_step(ready, input);
            (SpacesState::Ready(ready), effects)
        }
        (state, _) => (state, vec![]),
    }
}

fn ready_step(mut ready: SpacesReady, input: SpacesInput) -> (SpacesReady, Vec<SpacesEffect>) {
    let mut effects = Vec::new();
    match input {
        SpacesInput::AskRemove(key) if ready.removing.is_none() => {
            ready.sheet = ready
                .rows
                .iter()
                .find(|row| row.id == key)
                .map(|row| RemovalSheet::of(row.id.clone(), row.name.clone()));
        }
        SpacesInput::Sheet(change) => {
            ready.sheet = ready.sheet.map(|sheet| sheet.after(change));
        }
        SpacesInput::Confirm => {
            if let Some(sheet) = ready.sheet.take() {
                effects.push(SpacesEffect::Remove {
                    space: sheet.space.clone(),
                    removal: sheet.removal(),
                });
                ready.removing = Some(sheet.space);
            }
        }
        SpacesInput::Cancel => ready.sheet = None,
        SpacesInput::Removed(Ok(())) => {
            ready.removing = None;
            effects.push(SpacesEffect::Reload);
        }
        SpacesInput::Removed(Err(why)) => {
            ready.removing = None;
            effects.push(SpacesEffect::Tell(why));
            effects.push(SpacesEffect::Reload);
        }
        SpacesInput::AskRemove(_) | SpacesInput::Found(_) => {}
    }
    (ready, effects)
}
