//! How a sign-in the person started from Settings ended, as the module announces it on the
//! account's `reauth` row.

use ds_settings::schema::KeyPath;

/// What the module says the sign-in came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReauthEnd {
    /// Signed in again.
    Done,
    /// The person closed the sign-in.
    Cancelled,
    /// The service said no.
    Refused,
    /// An assistant's app was not running to sign it in.
    NoLauncher,
    /// Anything else: the service could not be reached, the sign-in could not be kept.
    Failed,
}

impl ReauthEnd {
    /// The end the module's word names, if it is one of the five.
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "done" => Self::Done,
            "cancelled" => Self::Cancelled,
            "refused" => Self::Refused,
            "no_launcher" => Self::NoLauncher,
            "failed" => Self::Failed,
            _ => return None,
        })
    }
}

/// Whether `path` is an account's `reauth` row.
pub(super) fn is_reauth_row(path: &KeyPath) -> bool {
    path.0.starts_with("accounts.") && path.0.ends_with(".reauth")
}
