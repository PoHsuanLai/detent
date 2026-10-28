//! The first-run step "Sign in to your accounts": a hidden page, drawn as a short dialogue, that
//! lists the accounts already signed in, offers the add sheet, and has Skip and Continue. Like the
//! assistant's step it is reached by its slug and never listed.

use super::model::{AccountKey, AccountProvider, AccountStatus, AccountsPane};
use super::module::ModuleId;
use crate::assistant_model::{SetupAction, SetupAvailability};
use porter_sheet::ProviderGroup;

/// The step's slug: what `--page` and `open-page` take.
pub const ACCOUNTS_SETUP_SLUG: &str = "accounts-setup";

/// An account the step lists as already signed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedIn {
    /// Its id.
    pub key: AccountKey,
    /// What the module calls it.
    pub label: String,
    /// The kind of account it is, for its mark and its second line.
    pub provider: Option<AccountProvider>,
    /// How it stands: one that needs a new sign-in is listed and says so.
    pub status: AccountStatus,
}

/// The accounts the step lists: the person's Internet accounts, in the pane's order. The assistants
/// and AI keys belong to the assistant's step, and the sync service's accounts to Sync.
pub fn signed_in(pane: &AccountsPane) -> Vec<SignedIn> {
    pane.in_group(ProviderGroup::Internet)
        .filter(|account| account.module != ModuleId::Syncd)
        .map(|account| SignedIn {
            key: account.key.clone(),
            label: account.label.clone(),
            provider: account.provider.clone(),
            status: account.status.clone(),
        })
        .collect()
}

/// Whether `button` can be pressed: Skip always; Continue once an account is listed.
pub fn accounts_setup_offer(listed: &[SignedIn], button: SetupAction) -> SetupAvailability {
    match (button, listed.is_empty()) {
        (SetupAction::Skip, _) | (SetupAction::Continue, false) => SetupAvailability::Now,
        (SetupAction::Continue, true) => SetupAvailability::NotYet,
    }
}

#[cfg(test)]
mod tests {
    use super::{ACCOUNTS_SETUP_SLUG, SignedIn, accounts_setup_offer, signed_in};
    use crate::accounts::tests::described;
    use crate::accounts::{AccountsPane, ModuleId};
    use crate::assistant_model::{SetupAction, SetupAvailability};
    use crate::page::PageSlug;
    use crate::page_kind::PageKind;

    #[test]
    fn the_step_lists_the_internet_accounts_and_continue_waits_for_one() {
        let pane = AccountsPane::of(ModuleId::Accountd, &described());
        let listed = signed_in(&pane);
        let labels: Vec<&str> = listed.iter().map(|row| row.label.as_str()).collect();
        assert_eq!(labels, ["Home cloud", "Work mail"]);
        let cases = [
            (SetupAction::Skip, true, SetupAvailability::Now),
            (SetupAction::Skip, false, SetupAvailability::Now),
            (SetupAction::Continue, false, SetupAvailability::NotYet),
            (SetupAction::Continue, true, SetupAvailability::Now),
        ];
        for (button, any, want) in cases {
            let shown: &[SignedIn] = if any { &listed[..] } else { &[] };
            assert_eq!(
                accounts_setup_offer(shown, button),
                want,
                "{button:?} {any}"
            );
        }
        assert!(PageKind::is_accounts_setup(&PageSlug(
            ACCOUNTS_SETUP_SLUG.to_owned()
        )));
        assert!(!PageKind::is_accounts_setup(&PageSlug(
            "accounts".to_owned()
        )));
    }
}
