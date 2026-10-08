//! How an account is signed in to again, which decides what the page tells the person to do.

use super::model::Account;

/// What signing in again asks of the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInWay {
    /// A provider's own page, opened in the browser (an account whose sign-in expires is one).
    Browser,
    /// The account's password, typed again (an IMAP or other password account). accountd does
    /// not yet say which accounts these are, so nothing builds it from the rows: `of` is where it
    /// will.
    Password,
    /// accountd does not say, so the words promise neither a browser nor a password.
    Unstated,
}

impl SignInWay {
    /// The way `account` is signed in to again, as far as its rows say. An expiry row is only
    /// there for a sign-in that a provider ends, which is a browser sign-in.
    pub fn of(account: &Account) -> SignInWay {
        match account.expires {
            Some(_) => SignInWay::Browser,
            None => SignInWay::Unstated,
        }
    }

    /// The line under the Sign in again row.
    pub fn help(self) -> &'static str {
        match self {
            SignInWay::Browser => "Sign in to the account again in your browser.",
            SignInWay::Password => "Enter the account's password again.",
            SignInWay::Unstated => "Sign in to the account again.",
        }
    }
}
