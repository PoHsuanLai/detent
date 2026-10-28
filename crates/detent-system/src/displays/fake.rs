//! The compositor's displays from memory: nothing here reaches a compositor. It writes down what
//! was asked and moves its own standing the way the real one would.

use super::DisplaysLink;
use crate::fake_kit::Kit;
use crate::ticks::Ticks;
use detent_model::{
    Absence, DisplayChange, DisplayEdit, DisplayMode, DisplaysStatus, Work, mode_at,
};
use std::sync::Arc;

/// Displays that answer from what they were given.
#[derive(Debug, Clone)]
pub struct FakeDisplays(Arc<Kit<DisplaysStatus, DisplayChange>>);

impl FakeDisplays {
    /// Displays in this standing.
    pub fn answering(status: DisplaysStatus) -> FakeDisplays {
        FakeDisplays(Kit::new(Ok(status)))
    }

    /// No compositor, for this reason.
    pub fn missing(why: Absence) -> FakeDisplays {
        FakeDisplays(Kit::new(Err(why)))
    }

    /// The next changes are refused, in these words.
    pub fn refuses(&self, words: &str) {
        self.0.refuse_with(Some(words.to_owned()));
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<DisplayChange> {
        self.0.asked()
    }
}

impl DisplaysLink for FakeDisplays {
    fn read(&self) -> Work<'_, Result<DisplaysStatus, Absence>> {
        Box::pin(std::future::ready(self.0.standing()))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        Box::pin(std::future::ready(Ok(self.0.listen())))
    }

    fn apply(&self, change: DisplayChange) -> Work<'_, Result<(), String>> {
        self.0.record(change.clone());
        let done = self.0.admit();
        if done.is_ok() {
            self.0.edit(|status| apply_to(status, &change));
        }
        Box::pin(std::future::ready(done))
    }
}

/// Moves the display `change` names the way the compositor would.
fn apply_to(status: &mut DisplaysStatus, change: &DisplayChange) {
    for display in status.displays.iter_mut().filter(|d| d.id == change.output) {
        match &change.edit {
            DisplayEdit::Scale(scale) => display.scale = *scale,
            DisplayEdit::Rotate(rotation) => display.rotation = *rotation,
            DisplayEdit::Move(at) => display.position = *at,
            DisplayEdit::Size(size) => {
                display.current = mode_at(&display.modes, display.current, *size);
            }
            DisplayEdit::Refresh(rate) => {
                display.current = display.current.map(|mode| DisplayMode {
                    refresh: *rate,
                    preferred: false,
                    ..mode
                });
            }
        }
    }
}
