//! Why an account needs the person: the sign-in ended, or the provider will end it on a known day.

use super::day::CalendarDay;
use super::model::{Account, AccountStatus};

/// What is wrong with an account that its screen puts first. One at a time: a sign-in that has
/// ended says more than one that is about to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attention {
    /// The sign-in ended; the account does nothing until the person signs in again.
    SignInEnded,
    /// The provider signs the account out on `day`.
    SignInEnding {
        /// The day.
        day: CalendarDay,
    },
}

/// The day an `expires` row names: its last word is `YYYY-MM-DD` ("Google signs this account out
/// on 2026-10-14"); none for a row of another shape.
fn expiry_day(text: &str) -> Option<CalendarDay> {
    CalendarDay::parse(text.rsplit(' ').next()?)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::model::{AccountExpiry, AccountProvider};
    use crate::accounts::{AccountKey, AccountSpend, ModuleId, Place};
    use ds_settings::schema::KeyPath;
    use porter_sheet::ProviderGroup;

    /// Whether the account has a Sign in again row.
    enum Reauth {
        Offered,
        Missing,
    }

    fn account(status: AccountStatus, expires: Option<&str>, reauth: Reauth) -> Account {
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
            show_in_files: None,
            permissions: Vec::new(),
            reauth: match reauth {
                Reauth::Offered => Some((
                    KeyPath("accounts.work.reauth".to_owned()),
                    "Sign in again".to_owned(),
                )),
                Reauth::Missing => None,
            },
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
        let day = CalendarDay {
            month: 10,
            date: 14,
        };
        let cases = [
            (
                "ended",
                account(AccountStatus::NeedsReauth, ended, Reauth::Offered),
                Some(Attention::SignInEnded),
            ),
            (
                "ended without an expiry row",
                account(AccountStatus::NeedsReauth, None, Reauth::Offered),
                Some(Attention::SignInEnded),
            ),
            (
                "ending",
                account(AccountStatus::Ok, ending, Reauth::Offered),
                Some(Attention::SignInEnding { day }),
            ),
            (
                "ended wins over ending",
                account(AccountStatus::NeedsReauth, ending, Reauth::Offered),
                Some(Attention::SignInEnded),
            ),
            (
                "ending with no sign-in row",
                account(AccountStatus::Ok, ending, Reauth::Missing),
                None,
            ),
            (
                "an expiry without a date",
                account(
                    AccountStatus::Ok,
                    Some("Google signs this account out"),
                    Reauth::Offered,
                ),
                None,
            ),
            (
                "fine",
                account(AccountStatus::Ok, None, Reauth::Offered),
                None,
            ),
        ];
        for (name, account, want) in cases {
            assert_eq!(Attention::of(&account), want, "{name}");
        }
    }
}
