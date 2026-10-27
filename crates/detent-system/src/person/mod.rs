//! The person using this computer: their name and picture, for the card at the top of the
//! sidebar. Read from AccountsService on the system bus, else from the passwd file and the
//! `.face` picture in their home folder. Nothing here starts a service.

mod bus;
#[cfg(feature = "testing")]
mod fake;

pub use bus::OnAccounts;
#[cfg(feature = "testing")]
pub use fake::FakePerson;

use detent_model::{Person, Work};
use std::fmt::Debug;

/// What the sidebar's card asks.
pub trait PersonLink: Debug + Send + Sync {
    /// Who is using this computer, or none when nothing says (the card is then left out).
    fn who(&self) -> Work<'_, Option<Person>>;
}

/// Nobody is known: no card.
#[derive(Debug, Clone, Copy)]
pub struct NoPerson;

impl PersonLink for NoPerson {
    fn who(&self) -> Work<'_, Option<Person>> {
        Box::pin(std::future::ready(None))
    }
}
