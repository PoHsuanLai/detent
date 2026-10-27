//! The desktop-wide Spaces, over accountd's registry and memory's controls: the list, and the
//! removal that first tells memory what becomes of the Space's memories and history. Every call
//! goes to the owner of the service's name that was confirmed, never to the name.

mod bus;
#[cfg(feature = "testing")]
mod fake;

pub use bus::OnBus;
#[cfg(feature = "testing")]
pub use fake::FakeSpaces;

use almanac_core::Removal;
use detent_model::{Absence, SpaceKey, SpaceRow, Work};
use std::fmt::Debug;

/// What the Spaces page asks of the services.
pub trait SpacesLink: Debug + Send + Sync {
    /// The desktop-wide Spaces, in the order they were made.
    fn list(&self) -> Work<'_, Result<Vec<SpaceRow>, Absence>>;
    /// Removes `space` after telling memory what becomes of what it holds; the words that say why
    /// not.
    fn remove(&self, space: SpaceKey, removal: Removal) -> Work<'_, Result<(), String>>;
}
