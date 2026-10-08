//! How a coding agent's sign-out ended, as accountd announces it on the account's `sign_out`
//! row, and the line the person reads for it.

use super::step::AccountsInput;
use ds_settings::schema::KeyPath;

/// What accountd says the sign-out came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignOutEnd {
    /// The agent's launcher was asked and has not answered yet.
    Asked,
    /// The agent signs itself in and out; Detent could not touch its login.
    LoginUntouched,
    /// The agent signed out.
    Done,
    /// The agent tried and could not.
    Failed,
    /// The person turned it down in the agent.
    Cancelled,
    /// The launcher did not answer in time.
    Expired,
    /// The app that runs the agent quit before it signed out.
    LauncherGone,
}

impl SignOutEnd {
    /// The end accountd's word names, if it is one of the seven.
    pub fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "asked" => Self::Asked,
            "login_untouched" => Self::LoginUntouched,
            "done" => Self::Done,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            "expired" => Self::Expired,
            "launcher_gone" => Self::LauncherGone,
            _ => return None,
        })
    }

    /// The line the person reads for the account called `label`.
    pub fn line(self, label: &str) -> String {
        match self {
            Self::Asked => format!("Asking {label} to sign out\u{2026}"),
            Self::LoginUntouched => {
                format!("{label} is still signed in on its own; open it to sign out there.")
            }
            Self::Done => format!("Signed out of {label}."),
            Self::Failed => format!("Couldn't sign out of {label}."),
            Self::Cancelled => format!("Signing out of {label} was cancelled."),
            Self::Expired => format!("{label} didn't answer in time."),
            Self::LauncherGone => {
                format!("The app that runs {label} quit before signing out.")
            }
        }
    }

    /// Whether the account may have changed, so the pane reads it again.
    pub(crate) fn settles(self) -> bool {
        !matches!(self, Self::Asked)
    }
}

impl AccountsInput {
    /// What the module's announcement of `value` at `path` means to the pane: a sign-out's end
    /// when it is on an account's `sign_out` row and says one of the seven words, else a change
    /// to read again.
    pub fn heard(path: KeyPath, value: &toml::Value) -> Self {
        let end = path
            .0
            .ends_with(".sign_out")
            .then(|| value.as_str().and_then(SignOutEnd::parse))
            .flatten();
        match end {
            Some(end) => Self::SignedOut { path, end },
            None => Self::Changed,
        }
    }
}
