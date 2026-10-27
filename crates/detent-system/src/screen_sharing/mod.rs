//! The screen-sharing portal behind the Privacy page's list of apps that may share the screen
//! without asking again, and taking one back.

mod bus;
#[cfg(feature = "testing")]
mod fake;
mod proxy;

pub use bus::OnBus;
#[cfg(feature = "testing")]
pub use fake::FakeScreenSharing;

use detent_model::{Absence, ProgramSubject, ScreenGrant, Work};
use std::fmt::Debug;

/// What the Privacy page asks of the portal.
pub trait ScreenSharingLink: Debug + Send + Sync {
    /// The apps that may share the screen without asking again. A portal nobody owns yet is asked
    /// to start first.
    fn list(&self) -> Work<'_, Result<Vec<ScreenGrant>, Absence>>;
    /// Takes `app` back, so the portal asks the next time; the words that say why not.
    fn revoke(&self, app: ProgramSubject) -> Work<'_, Result<(), String>>;
}
