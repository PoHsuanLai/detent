//! Where Settings is asked to open in the Accounts pane: an account's object path, read as the
//! segment accountd names it by.

use super::model::{AccountKey, AccountsPane};
use porter_core::AccountId;
use porter_ids::object_segment;

/// The object path every account lives under (`org.quire.Accounts1`'s `account/<segment>`).
/// accountd's `porter_dbus::accounts::account_path` is the writer of this; detent-model may not depend
/// on porter-dbus, so the prefix is repeated here and nowhere else.
const ACCOUNT_PREFIX: &str = "/org/quire/Accounts1/account/";

/// The last part of an account's object path: its id with every character that is not a letter
/// or digit turned into `_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSegment(pub String);

/// What a request to show an account asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountTarget {
    /// The Accounts list (no account, or a path that is not an account's).
    List,
    /// One account's detail.
    Detail(AccountSegment),
}

impl AccountTarget {
    /// The target an object path names: an account's path is its detail, anything else the list.
    pub fn of_path(path: &str) -> AccountTarget {
        match path.strip_prefix(ACCOUNT_PREFIX) {
            Some(segment)
                if !segment.is_empty()
                    && segment
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_') =>
            {
                AccountTarget::Detail(AccountSegment(segment.to_owned()))
            }
            _ => AccountTarget::List,
        }
    }
}

impl AccountsPane {
    /// The account whose object-path segment is `segment`, if the pane lists one.
    pub fn key_of(&self, segment: &AccountSegment) -> Option<AccountKey> {
        self.accounts
            .iter()
            .find(|account| {
                AccountId::parse(&account.key.0)
                    .is_ok_and(|id| object_segment(id.as_str()) == segment.0)
            })
            .map(|account| account.key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountSegment, AccountTarget};

    fn detail(segment: &str) -> AccountTarget {
        AccountTarget::Detail(AccountSegment(segment.to_owned()))
    }

    #[test]
    fn an_account_path_is_its_detail() {
        assert_eq!(
            AccountTarget::of_path("/org/quire/Accounts1/account/work_example"),
            detail("work_example")
        );
    }

    #[test]
    fn anything_else_is_the_list() {
        for path in [
            "",
            "/org/quire/Accounts1/account/",
            "/org/quire/Accounts1/account/a/b",
            "/org/quire/Accounts1/settings",
            "/elsewhere/account/home",
            "work.example",
        ] {
            assert_eq!(AccountTarget::of_path(path), AccountTarget::List, "{path}");
        }
    }
}
