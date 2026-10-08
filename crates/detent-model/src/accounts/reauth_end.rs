//! How a sign-in the person started from Settings ended, as the module announces it on the
//! account's `reauth` row, and the calm line the person reads for it.

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

    /// The line the person reads for the account called `label`.
    pub fn line(self, label: &str) -> String {
        match self {
            Self::Done => format!("Signed in to {label} again."),
            Self::Cancelled => format!("Signing in to {label} was cancelled."),
            Self::Refused => format!("The sign-in to {label} was refused."),
            Self::NoLauncher => {
                format!("The app that runs {label} is not open. Open it and try again.")
            }
            Self::Failed => format!("Could not sign in to {label}. Try again later."),
        }
    }
}

/// Whether `path` is an account's `reauth` row.
pub(super) fn is_reauth_row(path: &KeyPath) -> bool {
    path.0.starts_with("accounts.") && path.0.ends_with(".reauth")
}
