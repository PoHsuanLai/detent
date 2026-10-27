//! The permissions an app holds for an account: the row that revokes one, and the two read-only
//! rows beside it that say when it was given and how long it holds.

use super::day::CalendarDay;
use super::model::Account;
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
    /// The day it was given (UTC), from the `since` row; none when the row is missing or is not
    /// a day.
    pub since: Option<CalendarDay>,
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
}

impl AccountPermission {
    /// The app and the service the permission's sentence names; none for a sentence of another
    /// shape.
    pub fn sentence(&self) -> Option<GrantSentence> {
        GrantSentence::parse(&self.label)
    }
}

/// What the module decided for the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantDecision {
    /// The app may use the service.
    Allowed,
    /// The app may not.
    Refused,
}

/// A permission's sentence, taken apart: "Companion can use Language model" or "Mail can't use
/// Language model (this session)". The module words the row this way and publishes no separate
/// rows for the app or the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantSentence {
    /// The app, as the sentence names it.
    pub app: String,
    /// Whether it may.
    pub decision: GrantDecision,
    /// The service, as the sentence names it.
    pub service: String,
}

/// The words after the service in a permission that lasts one session.
const SESSION_TAIL: &str = " (this session)";

impl GrantSentence {
    fn parse(label: &str) -> Option<GrantSentence> {
        let label = label.strip_suffix(SESSION_TAIL).unwrap_or(label);
        [
            (" can use ", GrantDecision::Allowed),
            (" can't use ", GrantDecision::Refused),
        ]
        .into_iter()
        .find_map(|(middle, decision)| {
            let (app, service) = label.split_once(middle)?;
            (!app.is_empty() && !service.is_empty()).then(|| GrantSentence {
                app: app.to_owned(),
                decision,
                service: service.to_owned(),
            })
        })
    }
}

impl Account {
    /// The services the permissions of `app` allow, in the order the permissions are listed.
    pub fn services_of_app(&self, app: &str) -> Vec<String> {
        self.permissions
            .iter()
            .filter_map(AccountPermission::sentence)
            .filter(|sentence| sentence.app == app && sentence.decision == GrantDecision::Allowed)
            .map(|sentence| sentence.service)
            .collect()
    }
}

/// What a read-only row beside a permission says about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GrantFact {
    /// The day it was given.
    Since(CalendarDay),
    /// How long it holds.
    Scope(GrantScope),
}

impl AccountPermission {
    /// Takes in what a row beside the permission said.
    pub(super) fn learn(&mut self, fact: &GrantFact) {
        match fact {
            GrantFact::Since(day) => self.since = Some(*day),
            GrantFact::Scope(scope) => self.scope = Some(*scope),
        }
    }
}

/// The permission's row for grant `grant` of `account`: `accounts.<id>.grant.<grant>`.
pub(super) fn grant_path(account: &super::model::AccountKey, grant: &str) -> KeyPath {
    KeyPath(format!("accounts.{}.grant.{grant}", account.id()))
}

#[cfg(test)]
mod tests {
    use super::{AccountPermission, GrantDecision};
    use ds_settings::schema::KeyPath;

    fn permission(label: &str) -> AccountPermission {
        AccountPermission {
            path: KeyPath("accounts.key.grant.g1".to_owned()),
            label: label.to_owned(),
            action: "Revoke".to_owned(),
            since: None,
            scope: None,
        }
    }

    #[test]
    fn the_sentence_names_the_app_and_the_service() {
        let said = permission("Companion can use Language model (this session)").sentence();
        let said = said.expect("a sentence");
        assert_eq!(
            (said.app.as_str(), said.decision, said.service.as_str()),
            ("Companion", GrantDecision::Allowed, "Language model")
        );
        assert!(permission("Photos allowed: llm").sentence().is_none());
        let refused = permission("Mail can't use Language model").sentence();
        assert_eq!(
            refused.map(|said| said.decision),
            Some(GrantDecision::Refused)
        );
    }
}
