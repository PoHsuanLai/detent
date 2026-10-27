//! The Spaces registry and memory from memory: the list, and each removal that was asked.

use super::SpacesLink;
use crate::fake_kit::Kit;
use almanac_core::Removal;
use detent_model::{Absence, SpaceKey, SpaceRow, Work};
use std::sync::Arc;

/// Spaces that answer from what they were given.
#[derive(Debug, Clone)]
pub struct FakeSpaces(Arc<Kit<Vec<SpaceRow>, (SpaceKey, Removal)>>);

impl FakeSpaces {
    /// A registry holding these Spaces.
    pub fn holding(rows: Vec<SpaceRow>) -> FakeSpaces {
        FakeSpaces(Kit::new(Ok(rows)))
    }

    /// No registry, for this reason.
    pub fn missing(why: Absence) -> FakeSpaces {
        FakeSpaces(Kit::new(Err(why)))
    }

    /// The next removals are refused, in these words.
    pub fn refuses(&self, words: &str) {
        self.0.refuse_with(Some(words.to_owned()));
    }

    /// The removals that were asked, in order.
    pub fn removals(&self) -> Vec<(SpaceKey, Removal)> {
        self.0.asked()
    }
}

impl SpacesLink for FakeSpaces {
    fn list(&self) -> Work<'_, Result<Vec<SpaceRow>, Absence>> {
        Box::pin(std::future::ready(self.0.standing()))
    }

    fn remove(&self, space: SpaceKey, removal: Removal) -> Work<'_, Result<(), String>> {
        self.0.record((space.clone(), removal));
        let done = self.0.admit();
        if done.is_ok() {
            self.0.edit(|rows| rows.retain(|row| row.id != space));
        }
        Box::pin(std::future::ready(done))
    }
}
