//! What the module's announcement of a value at a row means to the pane.

use super::reauth_end::{ReauthEnd, is_reauth_row};
use super::sign_out::SignOutEnd;
use super::step::AccountsInput;
use ds_settings::schema::KeyPath;

impl AccountsInput {
    /// What the module's announcement of `value` at `path` means to the pane: a sign-out's end
    /// when it is on an account's `sign_out` row and says one of the seven words, a sign-in's end
    /// when it is on a `reauth` row and says one of the five, else a change to read again (which
    /// also covers a state change: the rows an account has depend on its state).
    pub fn heard(path: KeyPath, value: &toml::Value) -> Self {
        let word = value.as_str();
        let sign_out = path
            .0
            .ends_with(".sign_out")
            .then(|| word.and_then(SignOutEnd::parse))
            .flatten();
        if let Some(end) = sign_out {
            return Self::SignedOut { path, end };
        }
        let sign_in = is_reauth_row(&path)
            .then(|| word.and_then(ReauthEnd::parse))
            .flatten();
        match sign_in {
            Some(end) => Self::SignedIn { path, end },
            None => Self::Changed,
        }
    }
}
