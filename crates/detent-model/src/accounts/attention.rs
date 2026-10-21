//! Why an account needs the person, and the words that say so: the sign-in ended, or the provider
//! will end it on a known day.

use super::grant::day_words;
use super::model::{Account, AccountStatus};

/// What is wrong with an account that its screen puts first. One at a time: a sign-in that has
/// ended says more than one that is about to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attention {
    /// The sign-in ended; the account does nothing until the person signs in again.
    SignInEnded,
    /// The provider signs the account out on `day`, in the person's date words ("14 Oct").
    SignInEnding {
        /// The day, as words.
        day: String,
    },
}

/// The day an `expires` row names: its last word is `YYYY-MM-DD` ("Google signs this account out
/// on 2026-10-14"), read as the date words; none for a row of another shape.
fn expiry_day(text: &str) -> Option<String> {
    day_words(text.rsplit(' ').next()?)
}

impl Attention {
    /// What `account` needs, or none when it needs nothing or has no row to sign in again with.
    pub fn of(account: &Account) -> Option<Attention> {
        account.reauth.as_ref()?;
        if account.status == AccountStatus::NeedsReauth {
            return Some(Attention::SignInEnded);
        }
        let day = expiry_day(&account.expires.as_ref()?.text)?;
        Some(Attention::SignInEnding { day })
    }

    /// The sentence of the banner.
    pub fn words(&self, account: &Account) -> String {
        match self {
            Attention::SignInEnded => format!("Sign in again to keep using {}.", account.label),
            Attention::SignInEnding { day } => {
                let provider = account
                    .provider
                    .as_ref()
                    .map_or("The provider", |provider| provider.name.as_str());
                format!("{provider} will sign this account out on {day}.")
            }
        }
    }

    /// The name of the dot on the account's row.
    pub fn badge_label(&self) -> &'static str {
        match self {
            Attention::SignInEnded => "Needs sign-in",
            Attention::SignInEnding { .. } => "Sign-in ends soon",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::model::{AccountExpiry, AccountProvider};
    use crate::accounts::{AccountKey, AccountSpend, ModuleId, Place};
    use ds_settings::schema::KeyPath;
    use porter_core::sheet::ProviderGroup;

    fn account(status: AccountStatus, expires: Option<&str>, reauth: bool) -> Account {
        Account {
            key: AccountKey("work".to_owned()),
            module: ModuleId::Accountd,
            label: "Work mail".to_owned(),
            status,
            place: Place::Elsewhere,
            expires: expires.map(|text| AccountExpiry {
                label: "Sign-in".to_owned(),
                help: String::new(),
                text: text.to_owned(),
            }),
            sign_in: None,
            provider: Some(AccountProvider {
                id: "google".to_owned(),
                name: "Google".to_owned(),
            }),
            group: ProviderGroup::Internet,
            services: Vec::new(),
            sync: Vec::new(),
            permissions: Vec::new(),
            reauth: reauth.then(|| {
                (
                    KeyPath("accounts.work.reauth".to_owned()),
                    "Sign in again".to_owned(),
                )
            }),
            sign_out: None,
            remove: None,
            used_by: Vec::new(),
            spend: AccountSpend::default(),
        }
    }

    #[test]
    fn attention_follows_the_status_and_the_expiry_row() {
        let ending = Some("Google signs this account out on 2026-10-14");
        let ended = Some("Signed out by Google on 2026-10-14");
        let cases = [
            (
                "ended",
                account(AccountStatus::NeedsReauth, ended, true),
                Some("Sign in again to keep using Work mail."),
            ),
            (
                "ended without an expiry row",
                account(AccountStatus::NeedsReauth, None, true),
                Some("Sign in again to keep using Work mail."),
            ),
            (
                "ending",
                account(AccountStatus::Ok, ending, true),
                Some("Google will sign this account out on 14 Oct."),
            ),
            (
                "ended wins over ending",
                account(AccountStatus::NeedsReauth, ending, true),
                Some("Sign in again to keep using Work mail."),
            ),
            (
                "ending with no sign-in row",
                account(AccountStatus::Ok, ending, false),
                None,
            ),
            (
                "an expiry without a date",
                account(
                    AccountStatus::Ok,
                    Some("Google signs this account out"),
                    true,
                ),
                None,
            ),
            ("fine", account(AccountStatus::Ok, None, true), None),
        ];
        for (name, account, want) in cases {
            let got = Attention::of(&account).map(|a| a.words(&account));
            assert_eq!(got.as_deref(), want, "{name}");
        }
    }

    #[test]
    fn a_missing_provider_is_named_in_general_words() {
        let mut account = account(
            AccountStatus::Ok,
            Some("Google signs this account out on 2026-01-02"),
            true,
        );
        account.provider = None;
        let words = Attention::of(&account).map(|a| a.words(&account));
        assert_eq!(
            words.as_deref(),
            Some("The provider will sign this account out on 2 Jan.")
        );
    }
}
