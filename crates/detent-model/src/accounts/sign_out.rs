//! How an assistant's sign-out ended, as the module announces it on the account's `sign_out`
//! row, and the line the person reads for it.

/// What the module says the sign-out came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignOutEnd {
    /// The assistant's app was asked and has not answered yet.
    Asked,
    /// The assistant signs itself in and out; Settings could not touch its login.
    LoginUntouched,
    /// The assistant signed out.
    Done,
    /// The assistant tried and could not.
    Failed,
    /// The person turned it down in the assistant.
    Cancelled,
    /// The assistant's app did not answer in time.
    Expired,
    /// The app that runs the assistant quit before it signed out.
    LauncherGone,
}

impl SignOutEnd {
    /// The end the module's word names, if it is one of the seven.
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
