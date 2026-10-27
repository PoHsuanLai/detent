//! Adding an account from Settings: the "Add Account…" button asks the account service to open its
//! add sheet, and the sheet ends in one of these.

use super::model::AccountKey;

/// Whether an add sheet is open for this window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Adding {
    /// No sheet is open; the button can be pressed.
    #[default]
    Idle,
    /// A sheet is open; the button waits.
    SheetOpen,
}

/// How the add sheet ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddEnd {
    /// An account was added.
    Added,
    /// The person signed in to an account that was already there: it is this one.
    AlreadyThere(AccountKey),
    /// The person closed the sheet.
    Closed,
    /// This window already has an add sheet open.
    Busy,
    /// The account could not be added, or the service could not be reached.
    Failed,
}
