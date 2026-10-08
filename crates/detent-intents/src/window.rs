//! The window, as the provider needs it: told what happened, never asked.

use detent_model::{AccountTarget, PageSlug};

/// There is no window to tell: the app is closing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the Settings window has gone")]
pub struct WindowGone;

/// What the provider tells the window. The binary implements it over `detent-ui`'s outbox; a
/// test implements it by writing down what it was told.
pub trait Window: Send + Sync {
    /// Show `page` (opening the window if it is not open).
    fn open_page(&self, page: PageSlug) -> Result<(), WindowGone>;
    /// A settings file changed behind the window's back: read the files again.
    fn files_changed(&self) -> Result<(), WindowGone>;
    /// Show the Accounts page at `target` (an account's detail, or the list).
    fn open_account(&self, target: AccountTarget) -> Result<(), WindowGone>;
    /// Show the window and raise it; `token` is the xdg-activation token the caller handed over.
    fn raise(&self, token: Option<String>) -> Result<(), WindowGone>;
}
