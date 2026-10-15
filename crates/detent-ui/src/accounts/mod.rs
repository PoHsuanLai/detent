//! The account service, as the Accounts page meets it (PLAN section 2.5): the list of accounts, one
//! account's services, permissions and actions, and the sign-in keys, all over its live settings
//! module; and the sync service's held deletions. Every piece quire is still giving a controlled form (R2a) is drawn in
//! `adapter.rs` and nowhere else.

mod adapter;
mod add_io;
mod detail_view;
mod drive;
mod feeds;
mod follow_module;
mod hold_drive;
mod link;
mod list_view;
mod pane_view;
mod reload;
mod watch;

pub(crate) use adapter::OpenButton;
pub(crate) use add_io::add_account;
pub use drive::read_accounts;
pub(crate) use link::AccountLink;
pub(crate) use pane_view::AccountsPane;
pub(crate) use watch::{Listed, use_listed};
