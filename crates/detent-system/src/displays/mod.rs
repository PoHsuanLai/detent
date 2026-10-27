//! The compositor behind the Displays page: one handle the environment carries, so a test hands
//! in a compositor that only answers from memory.

mod brightness;
mod bus;
#[cfg(feature = "testing")]
mod fake;
mod map;

pub use bus::OnCompositor;
#[cfg(feature = "testing")]
pub use fake::FakeDisplays;

use crate::ticks::Ticks;
use detent_model::{
    Absence, Brightness, BrightnessList, DisplayChange, DisplaysStatus, OutputId, Work,
};
use std::fmt::Debug;

/// What the page needs of the compositor.
pub trait DisplaysLink: Debug + Send + Sync {
    /// The displays now, or why there are none.
    fn read(&self) -> Work<'_, Result<DisplaysStatus, Absence>>;
    /// A tick each time the displays may have changed, a change the compositor reverts included;
    /// only the compositor being unreachable is an error.
    fn changes(&self) -> Work<'_, Result<Ticks, Absence>>;
    /// Applies a change. The compositor asks the person whether to keep a change a program made,
    /// and goes back by itself when they do not; this answers when it has taken the change, not
    /// when it is kept. The compositor's words when it cannot.
    fn apply(&self, change: DisplayChange) -> Work<'_, Result<(), String>>;

    /// The displays that have a brightness to move, which are the ones the shell lists; none
    /// when there is no shell to ask.
    fn brightness(&self) -> Work<'_, Result<BrightnessList, Absence>> {
        Box::pin(std::future::ready(Err(Absence::NotRunning)))
    }
    /// A tick each time the shell says a brightness changed.
    fn brightness_changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        Box::pin(std::future::ready(Err(Absence::NotRunning)))
    }
    /// Sets one display's brightness. The words to say when it cannot.
    fn set_brightness(
        &self,
        _output: OutputId,
        _level: Brightness,
    ) -> Work<'_, Result<(), String>> {
        Box::pin(std::future::ready(Err(
            "The brightness can\u{2019}t be changed right now.".to_owned(),
        )))
    }
}
