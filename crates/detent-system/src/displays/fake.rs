//! The compositor's displays from memory: nothing here reaches a compositor. It writes down what
//! was asked and tells whoever follows it.

use super::DisplaysLink;
use crate::fake_kit::Kit;
use crate::ticks::Ticks;
use detent_model::{Absence, DisplayChange, DisplaysStatus, Work};
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
        self.0.record(change);
        self.0.tick();
        Box::pin(std::future::ready(self.0.admit()))
    }
}
