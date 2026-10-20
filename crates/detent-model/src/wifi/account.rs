//! What the person typed to sign in to a network that wants an account.

use super::passphrase::Passphrase;

/// A username and a password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// The name the network knows the person by.
    pub username: String,
    /// The password; its `Debug` hides the text.
    pub passphrase: Passphrase,
}

impl Account {
    /// The account `username` and `password` make; none while either is empty. The username is
    /// trimmed, the password is not.
    pub fn new(username: &str, password: &str) -> Option<Account> {
        let username = username.trim();
        (!username.is_empty() && !password.is_empty()).then(|| Account {
            username: username.to_owned(),
            passphrase: Passphrase::new(password),
        })
    }
}
