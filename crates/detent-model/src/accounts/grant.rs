//! The permissions an app holds for an account: the row that revokes one, and the two read-only
//! rows beside it that say when it was given and how long it holds.

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
}

impl AccountPermission {
    /// The line under the permission: when it was given, or that it lasts for the session.
    pub fn when_line(&self) -> Option<String> {
        let day = self.since.as_deref().and_then(day_words);
        let given = match self.sentence().map(|s| s.decision) {
            Some(GrantDecision::Refused) => "Blocked",
            Some(GrantDecision::Allowed) | None => "Allowed",
        };
        match (self.scope, day) {
            (Some(GrantScope::Session), _) => Some("This session".to_owned()),
            (Some(GrantScope::Once), Some(day)) => Some(format!("{given} once on {day}")),
            (Some(GrantScope::Once), None) => Some(format!("{given} once")),
            (Some(GrantScope::Always) | None, Some(day)) => Some(format!("{given} on {day}")),
            (Some(GrantScope::Always) | None, None) => None,
        }
    }

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

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// "8 Oct" for the day `YYYY-MM-DD`; none for text of another shape.
fn day_words(day: &str) -> Option<String> {
    let mut parts = day.split('-');
    let (_year, month, date) = (parts.next()?, parts.next()?, parts.next()?);
    let month: usize = month.parse().ok()?;
    let date: u8 = date.parse().ok()?;
    let name = MONTHS.get(month.checked_sub(1)?)?;
    (parts.next().is_none() && (1..=31).contains(&date)).then(|| format!("{date} {name}"))
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
    KeyPath(format!("accounts.{}.grant.{grant}", account.id()))
}

#[cfg(test)]
mod tests {
    use super::{AccountPermission, GrantDecision, GrantScope};
    use ds_settings::schema::KeyPath;

    fn permission(
        label: &str,
        since: Option<&str>,
        scope: Option<GrantScope>,
    ) -> AccountPermission {
        AccountPermission {
            path: KeyPath("accounts.key.grant.g1".to_owned()),
            label: label.to_owned(),
            action: "Revoke".to_owned(),
            since: since.map(str::to_owned),
            scope,
        }
    }

    #[test]
    fn a_permission_says_when_it_was_allowed_or_that_it_lasts_one_session() {
        const CASES: &[(&str, Option<&str>, Option<GrantScope>, Option<&str>)] = &[
            (
                "Mail can use Language model",
                Some("2026-10-08"),
                Some(GrantScope::Always),
                Some("Allowed on 8 Oct"),
            ),
            (
                "Mail can use Language model",
                Some("2026-12-31"),
                None,
                Some("Allowed on 31 Dec"),
            ),
            (
                "Mail can use Language model",
                Some("2026-10-08"),
                Some(GrantScope::Once),
                Some("Allowed once on 8 Oct"),
            ),
            (
                "Mail can use Language model (this session)",
                Some("2026-10-09"),
                Some(GrantScope::Session),
                Some("This session"),
            ),
            (
                "Mail can't use Language model",
                Some("2026-10-08"),
                Some(GrantScope::Always),
                Some("Blocked on 8 Oct"),
            ),
            (
                "Photos allowed: llm",
                Some("2026-10-08"),
                Some(GrantScope::Always),
                Some("Allowed on 8 Oct"),
            ),
            (
                "Mail can use Language model",
                Some("not a day"),
                Some(GrantScope::Always),
                None,
            ),
            ("Mail can use Language model", None, None, None),
        ];
        for (label, since, scope, want) in CASES {
            let got = permission(label, *since, *scope).when_line();
            assert_eq!(got.as_deref(), *want, "{label} {since:?} {scope:?}");
        }
    }

    #[test]
    fn the_sentence_names_the_app_and_the_service() {
        let said = permission(
            "Companion can use Language model (this session)",
            None,
            None,
        )
        .sentence();
        let said = said.expect("a sentence");
        assert_eq!(
            (said.app.as_str(), said.decision, said.service.as_str()),
            ("Companion", GrantDecision::Allowed, "Language model")
        );
        assert!(
            permission("Photos allowed: llm", None, None)
                .sentence()
                .is_none()
        );
    }
}
