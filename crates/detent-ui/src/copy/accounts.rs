//! The accounts pane's words: how an account stands, how it signs in, what needs the person, what
//! an app is allowed, and the question before an account is removed.

use detent_model::external::SignInWay;
use detent_model::{
    Account, AccountPermission, AccountService, AccountStatus, Ask, Attention, CalendarDay,
    GrantDecision, GrantScope, RemovalQuestion, humanize,
};

/// What a person reads for how an account stands.
pub(crate) fn status_label(status: &AccountStatus) -> String {
    match status {
        AccountStatus::Ok => "Signed in".to_owned(),
        AccountStatus::NeedsReauth | AccountStatus::NeedsLogin => "Needs sign-in".to_owned(),
        AccountStatus::Offline => "Offline".to_owned(),
        AccountStatus::Limited => "Limited".to_owned(),
        AccountStatus::Other(word) => humanize(word),
    }
}

/// The words a person reads for a service of an account.
pub(crate) fn service_label(service: &AccountService) -> String {
    humanize(&service.kind)
}

/// The line under the Sign in again row. An account whose row is missing, or says it signs in
/// outside the desktop's accounts or not at all, gets the plain line.
pub(crate) fn reauth_help(way: Option<SignInWay>) -> &'static str {
    match way {
        Some(SignInWay::Browser) => "Sign in to the account again in your browser.",
        Some(SignInWay::Password) => "Enter the password again.",
        Some(SignInWay::Key) => "Enter the key again.",
        Some(SignInWay::Agent) => {
            "Asks the assistant to sign itself in, which needs its app to be running."
        }
        Some(SignInWay::Outside | SignInWay::Nothing) | None => "Sign in to the account again.",
    }
}

/// The line under an account's name that says how it signs in; none when the row is missing or
/// says the account signs in outside the desktop's accounts or not at all.
pub(crate) fn sign_in_words(way: Option<SignInWay>) -> Option<&'static str> {
    match way? {
        SignInWay::Browser => Some("Signs in through the browser"),
        SignInWay::Password => Some("Signs in with a password"),
        SignInWay::Key => Some("Signs in with a key"),
        SignInWay::Agent => Some("Signs in inside its own app"),
        SignInWay::Outside | SignInWay::Nothing => None,
    }
}

/// The sentence of the banner at the head of an account that needs the person.
pub(crate) fn attention_words(attention: &Attention, account: &Account) -> String {
    match attention {
        Attention::SignInEnded => format!("Sign in again to keep using {}.", account.label),
        Attention::SignInEnding { day } => {
            let provider = account
                .provider
                .as_ref()
                .map_or("The provider", |provider| provider.name.as_str());
            format!(
                "{provider} will sign this account out on {}.",
                day_words(*day)
            )
        }
    }
}

/// The name of the dot on the account's row.
pub(crate) fn attention_badge(attention: &Attention) -> &'static str {
    match attention {
        Attention::SignInEnded => "Needs sign-in",
        Attention::SignInEnding { .. } => "Sign-in ends soon",
    }
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// "8 Oct".
fn day_words(day: CalendarDay) -> String {
    let month = MONTHS
        .get(usize::from(day.month).wrapping_sub(1))
        .copied()
        .unwrap_or_default();
    format!("{} {month}", day.date)
}

/// The line under a permission: when it was given, or that it lasts for the session.
pub(crate) fn grant_when(permission: &AccountPermission) -> Option<String> {
    let day = permission.since.map(day_words);
    let given = match permission.sentence().map(|sentence| sentence.decision) {
        Some(GrantDecision::Refused) => "Blocked",
        Some(GrantDecision::Allowed) | None => "Allowed",
    };
    match (permission.scope, day) {
        (Some(GrantScope::Session), _) => Some("This session".to_owned()),
        (Some(GrantScope::Once), Some(day)) => Some(format!("{given} once on {day}")),
        (Some(GrantScope::Once), None) => Some(format!("{given} once")),
        (Some(GrantScope::Always) | None, Some(day)) => Some(format!("{given} on {day}")),
        (Some(GrantScope::Always) | None, None) => None,
    }
}

/// The word a sign-in field shows in place of a secret it never reads back.
pub(crate) fn secret_standing(set: bool) -> &'static str {
    if set { "Set" } else { "Not set" }
}

/// What the alert before a removal or a sign-out says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemovalWords {
    /// The question.
    pub title: String,
    /// What it comes to.
    pub message: &'static str,
    /// The button that leaves things as they are.
    pub keep: &'static str,
    /// The button that does it.
    pub action: String,
}

/// The alert's words for `question`; `remove_name` is the module's name for the removal action.
pub(crate) fn removal(question: &RemovalQuestion, remove_name: &str) -> RemovalWords {
    match question.ask {
        Ask::Remove => RemovalWords {
            title: format!("{remove_name} {}?", question.account),
            message: if question.files_kept {
                "Its stored credentials and every permission given for it are deleted. \
                 Files kept on this computer for the account are deleted too. \
                 It cannot be undone."
            } else {
                "Its stored credentials and every permission given for it are deleted. \
                 It cannot be undone."
            },
            keep: "Keep",
            action: remove_name.to_owned(),
        },
        Ask::SignOut => RemovalWords {
            title: format!("Sign out of {}?", question.account),
            message: "You'll need to sign in again to use it.",
            keep: "Cancel",
            action: "Sign Out".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::external::ProviderGroup;
    use detent_model::{
        AccountKey, AccountProvider, AccountSpend, ModuleId, ModuleRow, Place, SwitchState,
    };
    use ds_settings::schema::KeyPath;

    fn account(status: AccountStatus, provider: Option<&str>) -> Account {
        Account {
            key: AccountKey("work".to_owned()),
            module: ModuleId::Accountd,
            label: "Work mail".to_owned(),
            status,
            place: Place::Elsewhere,
            expires: None,
            sign_in: None,
            provider: provider.map(|name| AccountProvider {
                id: "google".to_owned(),
                name: name.to_owned(),
            }),
            group: ProviderGroup::Internet,
            services: Vec::new(),
            sync: Vec::new(),
            show_in_files: None,
            permissions: Vec::new(),
            reauth: None,
            sign_out: None,
            remove: None,
            used_by: Vec::new(),
            spend: AccountSpend::default(),
        }
    }

    fn permission(
        label: &str,
        since: Option<CalendarDay>,
        scope: Option<GrantScope>,
    ) -> AccountPermission {
        AccountPermission {
            path: KeyPath("accounts.key.grant.g1".to_owned()),
            label: label.to_owned(),
            action: "Revoke".to_owned(),
            since,
            scope,
        }
    }

    #[test]
    fn an_account_stands_in_plain_words() {
        let cases = [
            (AccountStatus::Ok, "Signed in"),
            (AccountStatus::NeedsReauth, "Needs sign-in"),
            (AccountStatus::NeedsLogin, "Needs sign-in"),
            (AccountStatus::Offline, "Offline"),
            (AccountStatus::Limited, "Limited"),
            (
                AccountStatus::Other("rate_limited".to_owned()),
                "Rate limited",
            ),
        ];
        for (status, want) in cases {
            assert_eq!(status_label(&status), want, "{status:?}");
        }
    }

    #[test]
    fn a_service_is_its_kind_in_words() {
        let service = |kind: &str| AccountService {
            path: KeyPath(format!("accounts.home.service.{kind}")),
            kind: kind.to_owned(),
            on: SwitchState::On,
        };
        for (kind, want) in [("storage", "Storage"), ("language_model", "Language model")] {
            assert_eq!(service_label(&service(kind)), want, "{kind}");
        }
    }

    #[test]
    fn each_way_of_signing_in_words_its_lines() {
        let cases = [
            (
                Some(SignInWay::Browser),
                "Sign in to the account again in your browser.",
                Some("Signs in through the browser"),
            ),
            (
                Some(SignInWay::Password),
                "Enter the password again.",
                Some("Signs in with a password"),
            ),
            (
                Some(SignInWay::Key),
                "Enter the key again.",
                Some("Signs in with a key"),
            ),
            (
                Some(SignInWay::Agent),
                "Asks the assistant to sign itself in, which needs its app to be running.",
                Some("Signs in inside its own app"),
            ),
            (
                Some(SignInWay::Outside),
                "Sign in to the account again.",
                None,
            ),
            (
                Some(SignInWay::Nothing),
                "Sign in to the account again.",
                None,
            ),
            (None, "Sign in to the account again.", None),
        ];
        for (way, help, words) in cases {
            assert_eq!(reauth_help(way), help, "{way:?}");
            assert_eq!(sign_in_words(way), words, "{way:?}");
        }
    }

    #[test]
    fn the_banner_says_what_the_account_needs() {
        let day = CalendarDay {
            month: 10,
            date: 14,
        };
        let cases = [
            (
                Attention::SignInEnded,
                account(AccountStatus::NeedsReauth, Some("Google")),
                "Sign in again to keep using Work mail.",
                "Needs sign-in",
            ),
            (
                Attention::SignInEnding { day },
                account(AccountStatus::Ok, Some("Google")),
                "Google will sign this account out on 14 Oct.",
                "Sign-in ends soon",
            ),
            (
                Attention::SignInEnding {
                    day: CalendarDay { month: 1, date: 2 },
                },
                account(AccountStatus::Ok, None),
                "The provider will sign this account out on 2 Jan.",
                "Sign-in ends soon",
            ),
        ];
        for (attention, account, words, badge) in cases {
            assert_eq!(attention_words(&attention, &account), words);
            assert_eq!(attention_badge(&attention), badge);
        }
    }

    #[test]
    fn a_permission_says_when_it_was_allowed_or_that_it_lasts_one_session() {
        let day = |month, date| Some(CalendarDay { month, date });
        let cases = [
            (
                "Mail can use Language model",
                day(10, 8),
                Some(GrantScope::Always),
                Some("Allowed on 8 Oct"),
            ),
            (
                "Mail can use Language model",
                day(12, 31),
                None,
                Some("Allowed on 31 Dec"),
            ),
            (
                "Mail can use Language model",
                day(10, 8),
                Some(GrantScope::Once),
                Some("Allowed once on 8 Oct"),
            ),
            (
                "Mail can use Language model",
                None,
                Some(GrantScope::Once),
                Some("Allowed once"),
            ),
            (
                "Mail can use Language model (this session)",
                day(10, 9),
                Some(GrantScope::Session),
                Some("This session"),
            ),
            (
                "Mail can't use Language model",
                day(10, 8),
                Some(GrantScope::Always),
                Some("Blocked on 8 Oct"),
            ),
            (
                "Photos allowed: llm",
                day(10, 8),
                Some(GrantScope::Always),
                Some("Allowed on 8 Oct"),
            ),
            (
                "Mail can use Language model",
                None,
                Some(GrantScope::Always),
                None,
            ),
            ("Mail can use Language model", None, None, None),
        ];
        for (label, since, scope, want) in cases {
            let got = grant_when(&permission(label, since, scope));
            assert_eq!(got.as_deref(), want, "{label} {since:?} {scope:?}");
        }
    }

    #[test]
    fn a_secret_says_whether_it_is_set() {
        for (set, want) in [(true, "Set"), (false, "Not set")] {
            assert_eq!(secret_standing(set), want);
        }
    }

    #[test]
    fn the_question_names_what_goes_with_the_account() {
        let asked = |ask, files_kept| RemovalQuestion {
            ask,
            row: ModuleRow::new(ModuleId::Accountd, KeyPath("accounts.a.remove".to_owned())),
            account: "Home cloud".to_owned(),
            files_kept,
        };
        let cases = [
            (
                asked(Ask::Remove, true),
                RemovalWords {
                    title: "Remove account Home cloud?".to_owned(),
                    message: "Its stored credentials and every permission given for it are \
                              deleted. Files kept on this computer for the account are deleted \
                              too. It cannot be undone.",
                    keep: "Keep",
                    action: "Remove account".to_owned(),
                },
            ),
            (
                asked(Ask::Remove, false),
                RemovalWords {
                    title: "Remove account Home cloud?".to_owned(),
                    message: "Its stored credentials and every permission given for it are \
                              deleted. It cannot be undone.",
                    keep: "Keep",
                    action: "Remove account".to_owned(),
                },
            ),
            (
                asked(Ask::SignOut, false),
                RemovalWords {
                    title: "Sign out of Home cloud?".to_owned(),
                    message: "You'll need to sign in again to use it.",
                    keep: "Cancel",
                    action: "Sign Out".to_owned(),
                },
            ),
        ];
        for (question, want) in cases {
            assert_eq!(removal(&question, "Remove account"), want, "{question:?}");
        }
    }
}
