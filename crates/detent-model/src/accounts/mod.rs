//! accountd's Accounts pane, the live module `org.quire.SettingsModule1` at
//! `/org/quire/Accounts1/settings` (PLAN section 2.5), as pure state: the rows read as accounts,
//! and the transition a person's choices make.

mod model;
mod step;

pub use model::{
    Account, AccountKey, AccountPermission, AccountService, AccountStatus, AccountSync,
    AccountsPane, AccountsReady, AccountsScreen, AccountsState, Ask, Place, RemovalQuestion,
    SignInClient,
};
pub use step::{AccountsEffect, AccountsInput, accounts_step};

#[cfg(test)]
mod tests;
