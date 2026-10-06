//! accountd, as the Accounts page meets it (PLAN section 2.5): the list of accounts, one
//! account's services, permissions and actions, and the sign-in client ids, all over accountd's
//! live settings module. Every piece quire is still giving a controlled form (R2a) is drawn in
//! `adapter.rs` and nowhere else.

mod adapter;
mod detail_view;
mod drive;
mod list_view;
mod pane_view;

pub use drive::read_accounts;
pub(crate) use pane_view::AccountsPane;
