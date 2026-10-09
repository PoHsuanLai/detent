//! The permissions an app holds for an account: the row that revokes one, and the two read-only
//! rows beside it that say when it was given and how long it holds.

use ds_settings::schema::KeyPath;

/// One permission an app holds for the account: a sentence and the action that takes it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountPermission {
    /// The row, `accounts.<id>.grant.<grant>`.
    pub path: KeyPath,
    /// What the app may or may not do.
    pub label: String,
    /// The action's name (`Revoke`).
    pub action: String,
    /// The day it was given, `YYYY-MM-DD` (UTC), from the `since` row; none when the row is
    /// missing.
    pub since: Option<String>,
    /// How long it holds, from the `scope` row; none when the row is missing.
    pub scope: Option<GrantScope>,
}

/// How long a permission holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantScope {
    /// Once: the next use asks again.
    Once,
    /// Until it is revoked.
    Always,
    /// Until the app's session ends.
    Session,
}

impl GrantScope {
    /// The scope a `scope` row's word names.
    pub(super) fn of(word: &str) -> Option<GrantScope> {
        match word {
            "once" => Some(GrantScope::Once),
            "always" => Some(GrantScope::Always),
            "session" => Some(GrantScope::Session),
            _ => None,
        }
    }

    /// What a person reads.
    pub fn label(self) -> &'static str {
        match self {
            GrantScope::Once => "Once",
            GrantScope::Always => "Always",
            GrantScope::Session => "This session only",
        }
    }
}

impl AccountPermission {
    /// The line under the permission: when it was given and how long it holds, whichever the
    /// module said.
    pub fn detail(&self) -> Option<String> {
        match (&self.since, self.scope) {
            (Some(day), Some(scope)) => Some(format!("Since {day}. {}", scope.label())),
            (Some(day), None) => Some(format!("Since {day}")),
            (None, Some(scope)) => Some(scope.label().to_owned()),
            (None, None) => None,
        }
    }
}

/// What a read-only row beside a permission says about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GrantFact {
    /// The day it was given.
    Since(String),
    /// How long it holds.
    Scope(GrantScope),
}

impl AccountPermission {
    /// Takes in what a row beside the permission said.
    pub(super) fn learn(&mut self, fact: &GrantFact) {
        match fact {
            GrantFact::Since(day) => self.since = Some(day.clone()),
            GrantFact::Scope(scope) => self.scope = Some(*scope),
        }
    }
}

/// The permission's row for grant `grant` of `account`: `accounts.<id>.grant.<grant>`.
pub(super) fn grant_path(account: &super::model::AccountKey, grant: &str) -> KeyPath {
    KeyPath(format!("accounts.{}.grant.{grant}", account.0))
}
