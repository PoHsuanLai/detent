//! What the Accounts pane asks its caller to do.

use super::add_account::AddEnd;
use super::model::AccountKey;
use super::module::{ModuleId, ModuleRow};
use super::reauth_end::ReauthEnd;
use super::sign_in_client::Secret;
use super::sign_out::SignOutEnd;

/// What the caller must do.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsEffect {
    /// Call `Set(path, value)` on the module.
    Set {
        /// The row.
        row: ModuleRow,
        /// The value.
        value: toml::Value,
    },
    /// Call `Set(path, text)` on the module with a sign-in secret, which is never read back.
    SetSecret {
        /// The row.
        row: ModuleRow,
        /// The secret.
        secret: Secret,
    },
    /// Run the action at the row.
    Invoke(ModuleRow),
    /// Open the add sheet for an identity and tell how it ended as `AddEnded`.
    OpenAddSheet,
    /// Open the add sheet for an AI key and tell how it ended as `AddEnded`.
    OpenAiKeySheet,
    /// Read the module again.
    Reload(ModuleId),
    /// Read the module again after a short wait: the last read failed.
    Retry(ModuleId),
    /// Open the account with this key once the rows are read again.
    Show(AccountKey),
    /// Tell the person something.
    Tell(AccountsNotice),
}

/// What the person is told, as the fact that caused it; the words are the window's.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsNotice {
    /// A secret was typed before its key.
    KeyFirst,
    /// The add sheet ended in a way the person is told about.
    AddEnded(AddEnd),
    /// A sign-in the person started ended; `account` is its label, none when the pane has no
    /// account for the row.
    SignedIn {
        /// How it ended.
        end: ReauthEnd,
        /// The account's label.
        account: Option<String>,
    },
    /// An assistant's sign-out ended; `account` is its label, none when the pane has no account
    /// for the row.
    SignedOut {
        /// How it ended.
        end: SignOutEnd,
        /// The account's label.
        account: Option<String>,
    },
    /// A module refused what was asked, in its own words.
    Refused(String),
}
