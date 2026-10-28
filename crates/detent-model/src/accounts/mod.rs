//! The account service's Accounts pane, the live module `org.quire.SettingsModule1` at
//! `/org/quire/Accounts1/settings` (PLAN section 2.5), as pure state: the rows read as accounts,
//! and the transition a person's choices make.

mod add_account;
mod attention;
mod day;
mod effect;
mod grant;
mod heard;
mod hold;
mod hold_step;
mod input;
mod model;
mod module;
mod pane;
mod reads;
mod reauth_end;
mod rows;
mod setup;
mod sign_in_client;
mod sign_out;
mod spend;
mod step;
mod sync_now;
mod sync_status;
mod target;
mod valued;

pub use add_account::AddEnd;
pub use attention::Attention;
pub use day::CalendarDay;
pub use effect::{AccountsEffect, AccountsNotice};
pub use grant::{AccountPermission, GrantDecision, GrantScope, GrantSentence};
pub use hold::{DatasetName, HoldCounts, SyncHold};
pub use hold_step::{HoldEffect, HoldInput, Holds, hold_step};
pub use input::AccountsInput;
pub use model::{
    Account, AccountExpiry, AccountKey, AccountProvider, AccountService, AccountStatus,
    AccountSync, AccountsPane, AccountsReady, AccountsState, Ask, Place, RemovalQuestion,
    SwitchState, SyncAdd,
};
pub use module::{ModuleId, ModuleRow, Source, Sources};
pub use reads::Reads;
pub use reauth_end::ReauthEnd;
pub use setup::{ACCOUNTS_SETUP_SLUG, SignedIn, accounts_setup_offer, signed_in};
pub use sign_in_client::{ClientText, ClientValue, Secret, SignInClient};
pub use sign_out::SignOutEnd;
pub use spend::{AccountSpend, Cents, SpendBar, SpendLine};
pub use step::accounts_step;
pub use sync_now::SyncNowEnd;
pub use sync_status::{StorageQuota, SyncBook, SyncStatus};
pub use target::{AccountSegment, AccountTarget};
pub use valued::accounts_row_is_valued;

#[cfg(test)]
mod client_tests;
#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod hold_tests;
#[cfg(test)]
mod modules_tests;
#[cfg(test)]
mod pane_step_tests;
#[cfg(test)]
mod reading_tests;
#[cfg(test)]
mod reread_tests;
#[cfg(test)]
mod sign_out_tests;
#[cfg(test)]
mod tests;
