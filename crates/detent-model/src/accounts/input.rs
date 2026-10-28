//! What can happen to the Accounts pane: what the modules told it and what the person did.

use super::add_account::AddEnd;
use super::model::{AccountKey, SwitchState};
use super::module::{ModuleId, ModuleRow};
use super::reauth_end::ReauthEnd;
use super::sign_in_client::ClientText;
use super::sign_out::SignOutEnd;
use crate::Absence;
use crate::picker::Described;

/// What happened to the pane.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsInput {
    /// A module was found: the first time it was read, or when it came back; or it is not there.
    Found(ModuleId, Result<Described, Absence>),
    /// A module was read again after a change; a failed read keeps what is shown.
    Reread(ModuleId, Result<Described, Absence>),
    /// A link asked for an account: it is chosen, as if the person had chosen it in the list.
    Open(AccountKey),
    /// The person chose an account in the list; its detail is beside the list.
    Select(AccountKey),
    /// Nothing is chosen: a link asked for the list.
    Back,
    /// The person turned a service on or off.
    Switched {
        /// The service's row.
        row: ModuleRow,
        /// What it is now.
        on: SwitchState,
    },
    /// The person pressed a permission's action or Sign in again.
    Ran(ModuleRow),
    /// The person asked to remove an account.
    AskRemove(ModuleRow),
    /// The person asked to sign an assistant's account out.
    AskSignOut(ModuleRow),
    /// The person answered the question: no.
    Keep,
    /// The person answered the question: yes.
    Remove,
    /// The person finished a sign-in client id.
    ClientEdited {
        /// The row.
        row: ModuleRow,
        /// What the field holds.
        text: ClientText,
    },
    /// The person pressed Clear beside a sign-in secret that is set.
    SecretCleared(ModuleRow),
    /// The module did what was asked at `path`.
    Done(ModuleRow),
    /// The module refused what was asked at `path`.
    Failed {
        /// The row.
        row: ModuleRow,
        /// What the module said.
        why: String,
    },
    /// The person pressed Add Account….
    AddAccount,
    /// The add sheet ended; it was the one of `module`.
    AddEnded(ModuleId, AddEnd),
    /// The module says something changed.
    Changed(ModuleId),
    /// The module says how the sign-in the person started at `path` ended.
    SignedIn {
        /// The account's reauth row.
        row: ModuleRow,
        /// How it ended.
        end: ReauthEnd,
    },
    /// The module says how an assistant's sign-out at `path` ended.
    SignedOut {
        /// The account's sign-out row.
        row: ModuleRow,
        /// How it ended.
        end: SignOutEnd,
    },
}
