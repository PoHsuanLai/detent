//! The account service's Accounts pane, the live module `org.quire.SettingsModule1` at
//! `/org/quire/Accounts1/settings` (PLAN section 2.5), as pure state: the rows read as accounts,
//! and the transition a person's choices make.

mod add_account;
mod grant;
mod heard;
mod hold;
mod hold_step;
mod model;
mod reauth_end;
mod sign_in_help;
mod sign_out;
mod step;
mod sync_now;
mod target;

pub use add_account::{AddEnd, Adding};
pub use grant::AccountPermission;
pub use hold::{DatasetName, HoldCounts, KEEP_HELP, SyncHold};
pub use hold_step::{HoldEffect, HoldInput, Holds, hold_step};
pub use model::{
    Account, AccountExpiry, AccountKey, AccountProvider, AccountService, AccountStatus,
    AccountSync, AccountsPane, AccountsReady, AccountsScreen, AccountsState, Ask, ClientEntry,
    LocalFiles, Place, RemovalQuestion, SignInClient,
};
pub use reauth_end::ReauthEnd;
pub use sign_in_help::reauth_help;
pub use sign_out::SignOutEnd;
pub use step::{AccountsEffect, AccountsInput, accounts_step};
pub use sync_now::SyncNowEnd;
pub use target::{AccountSegment, AccountTarget};

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod hold_tests;
#[cfg(test)]
mod tests;
