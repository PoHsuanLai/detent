//! The Displays page as a machine.

use super::arrange::{ArrangeSide, place_beside};
use super::model::{
    DisplayPosition, DisplayRotation, DisplaysStatus, OutputId, PixelSize, RefreshMilliHz,
    ScaleMilli,
};
use crate::Absence;
use crate::Live;

/// Where the page stands: the displays once read.
pub type DisplaysState = Live<DisplaysReady>;

/// The page with something to show.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplaysReady {
    /// What the compositor last said.
    pub status: DisplaysStatus,
    /// The display a change was sent for and has not ended.
    pub applying: Option<OutputId>,
}

/// One thing changed about a display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayEdit {
    /// A different scale, so a different resolution to look at.
    Scale(ScaleMilli),
    /// A different refresh rate at the same size.
    Refresh(RefreshMilliHz),
    /// A different mode size, at the refresh rate it runs now when the new size has it.
    Size(PixelSize),
    /// A different rotation.
    Rotate(DisplayRotation),
    /// A different place.
    Move(DisplayPosition),
}

/// A change to one display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayChange {
    /// The display.
    pub output: OutputId,
    /// What changes.
    pub edit: DisplayEdit,
}

/// What the page hears.
#[derive(Debug, Clone)]
pub enum DisplaysInput {
    /// A read of the compositor, or why there is none.
    Heard(Result<DisplaysStatus, Absence>),
    /// The person changed a display.
    Change(DisplayChange),
    /// The person put a display beside another.
    Arrange(OutputId, OutputId, ArrangeSide),
    /// A change ended, with the words to say if it failed.
    Applied(OutputId, Result<(), String>),
}

/// What the page asks of the compositor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplaysEffect {
    /// Apply a change. The compositor asks the person whether to keep it.
    Apply(DisplayChange),
    /// Say this to the person.
    Tell(String),
}

/// The next state, and what to ask of the compositor.
pub fn displays_step(
    state: DisplaysState,
    input: DisplaysInput,
) -> (DisplaysState, Vec<DisplaysEffect>) {
    match (state, input) {
        (state, DisplaysInput::Heard(heard)) => (heard_step(state, heard), Vec::new()),
        (DisplaysState::Ready(page), input) => {
            let (page, effects) = press_step(page, input);
            (DisplaysState::Ready(page), effects)
        }
        (other, _) => (other, Vec::new()),
    }
}

fn heard_step(state: DisplaysState, heard: Result<DisplaysStatus, Absence>) -> DisplaysState {
    let status = match heard {
        Ok(status) => status,
        Err(missing) => return DisplaysState::Unavailable(missing),
    };
    let applying = match state {
        DisplaysState::Ready(page) => page.applying,
        DisplaysState::Looking | DisplaysState::Unavailable(_) => None,
    };
    DisplaysState::Ready(DisplaysReady { status, applying })
}

fn press_step(page: DisplaysReady, input: DisplaysInput) -> (DisplaysReady, Vec<DisplaysEffect>) {
    match input {
        DisplaysInput::Heard(_) => (page, Vec::new()),
        DisplaysInput::Change(change) => apply(page, change),
        DisplaysInput::Arrange(output, beside, side) => arrange(page, output, &beside, side),
        DisplaysInput::Applied(output, result) => {
            let applying = page.applying.clone().filter(|applying| applying != &output);
            let effects = result.err().map(DisplaysEffect::Tell).into_iter().collect();
            (DisplaysReady { applying, ..page }, effects)
        }
    }
}

/// Sends a change, unless one is still under way: the compositor holds one trial at a time.
fn apply(page: DisplaysReady, change: DisplayChange) -> (DisplaysReady, Vec<DisplaysEffect>) {
    if page.applying.is_some() || page.status.display(&change.output).is_none() {
        return (page, Vec::new());
    }
    (
        DisplaysReady {
            applying: Some(change.output.clone()),
            ..page
        },
        vec![DisplaysEffect::Apply(change)],
    )
}

fn arrange(
    page: DisplaysReady,
    output: OutputId,
    beside: &OutputId,
    side: ArrangeSide,
) -> (DisplaysReady, Vec<DisplaysEffect>) {
    let place = match (page.status.display(&output), page.status.display(beside)) {
        (Some(display), Some(other)) => place_beside(display, other, side),
        _ => None,
    };
    match place {
        Some(at) => apply(
            page,
            DisplayChange {
                output,
                edit: DisplayEdit::Move(at),
            },
        ),
        None => (page, Vec::new()),
    }
}

impl DisplaysReady {
    /// Whether a change is under way.
    pub fn is_applying(&self) -> bool {
        self.applying.is_some()
    }
}
