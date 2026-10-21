//! The account service's Accounts pane, the live module `org.quire.SettingsModule1` at
//! `/org/quire/Accounts1/settings` (PLAN section 2.5), as pure state: the rows read as accounts,
//! and the transition a person's choices make.

mod add_account;
mod attention;
mod grant;
mod heard;
mod hold;
mod hold_step;
mod model;
mod module;
mod reads;
mod reauth_end;
mod setup;
mod sign_in_client;
mod sign_in_help;
mod sign_out;
mod spend;
mod step;
mod sync_now;
mod sync_status;
mod target;

pub use add_account::{AddEnd, Adding};
pub use attention::Attention;
pub use grant::{AccountPermission, GrantDecision, GrantSentence};
pub use hold::{DatasetName, HoldCounts, KEEP_HELP, SyncHold};
pub use hold_step::{HoldEffect, HoldInput, Holds, hold_step};
pub use model::{
    Account, AccountExpiry, AccountKey, AccountProvider, AccountService, AccountStatus,
    AccountSync, AccountsPane, AccountsReady, AccountsScreen, AccountsState, Ask, LocalFiles,
    Place, RemovalQuestion, SyncAdd,
};
pub use module::{ModuleId, ModuleRow, Source, Sources};
pub use reads::{ReadTurn, Reads};
pub use reauth_end::ReauthEnd;
pub use setup::{
    ACCOUNTS_SETUP_SLUG, SignedIn, accounts_setup_button, is_accounts_setup, signed_in,
};
pub use sign_in_client::{
    ClientEntry, ClientText, ClientValue, Secret, SecretStanding, SignInClient,
};
pub use sign_in_help::{reauth_help, sign_in_words};
pub use sign_out::SignOutEnd;
pub use spend::{AccountSpend, Cents, SpendBar, SpendLine};
pub use step::{AccountsEffect, AccountsInput, accounts_step};
pub use sync_now::SyncNowEnd;
pub use sync_status::{StorageQuota, SyncBook, SyncPause, SyncStatus, SyncWork};
pub use target::{AccountSegment, AccountTarget};

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod hold_tests;
#[cfg(test)]
mod modules_tests;
#[cfg(test)]
mod reread_tests;
#[cfg(test)]
mod tests;
