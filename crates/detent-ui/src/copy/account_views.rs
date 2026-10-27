//! The Accounts page's own words: its headings, its buttons and what it says when there is nothing to show.

/// The words "Add Account".
pub(crate) const ADD_ACCOUNT: &str = "Add Account";

/// The words "Add AI Key…".
pub(crate) const ADD_AI_KEY_ELLIPSIS: &str = "Add AI Key\u{2026}";

/// The words "Remove Account".
pub(crate) const REMOVE_ACCOUNT: &str = "Remove Account";

/// The words "{held} files removed on the server".
pub(crate) fn files_removed_on_the_server(held: impl std::fmt::Display) -> String {
    format!("{held} files removed on the server")
}

/// The words "Keep".
pub(crate) const KEEP: &str = "Keep";

/// The words "Sign In…".
pub(crate) const SIGN_IN_ELLIPSIS: &str = "Sign In\u{2026}";

/// The words "Syncing {label}".
pub(crate) fn syncing(label: impl std::fmt::Display) -> String {
    format!("Syncing {label}")
}

/// The words "Status".
pub(crate) const STATUS: &str = "Status";

/// The words "Services".
pub(crate) const SERVICES: &str = "Services";

/// The words "Whether apps may use each service of the account.".
pub(crate) const SERVICES_HELP: &str = "Whether apps may use each service of the account.";

/// The words "Spending".
pub(crate) const SPENDING: &str = "Spending";

/// The words "This month".
pub(crate) const THIS_MONTH: &str = "This month";

/// The words "Spent this month".
pub(crate) const SPENT_THIS_MONTH: &str = "Spent this month";

/// The words "Sync".
pub(crate) const SYNC: &str = "Sync";

/// The words "Apps with access".
pub(crate) const APPS_WITH_ACCESS: &str = "Apps with access";

/// The words "Revoking asks the app again the next time it needs the account.".
pub(crate) const REVOKE_HELP: &str =
    "Revoking asks the app again the next time it needs the account.";

/// The words "Apps using this identity".
pub(crate) const APPS_USING_THIS_IDENTITY: &str = "Apps using this identity";

/// The words "Sync now".
pub(crate) const SYNC_NOW: &str = "Sync now";

/// The words "Revoke \u{2192} asks again next time".
pub(crate) const REVOKE_ASKS_AGAIN_NEXT_TIME: &str = "Revoke \u{2192} asks again next time";

/// The words "Revoke {}".
pub(crate) fn revoke(a: impl std::fmt::Display) -> String {
    format!("Revoke {a}")
}

/// The words "Uses none of this account's services".
pub(crate) const USES_NO_SERVICE: &str = "Uses none of this account's services";

/// The words "Uses {}".
pub(crate) fn uses(a: impl std::fmt::Display) -> String {
    format!("Uses {a}")
}

/// The words "No accounts".
pub(crate) const NO_ACCOUNTS: &str = "No accounts";

/// The words "An app that needs an account asks for it, and it appears here.".
pub(crate) const NO_ACCOUNTS_HINT: &str =
    "An app that needs an account asks for it, and it appears here.";

/// The words "AI keys".
pub(crate) const AI_KEYS: &str = "AI keys";

/// The words "Add a key, or sign in to an assistant.".
pub(crate) const ADD_A_KEY_OR_SIGN_IN: &str = "Add a key, or sign in to an assistant.";

/// The words "Choose an account to sync.".
pub(crate) const CHOOSE_AN_ACCOUNT_TO_SYNC: &str = "Choose an account to sync.";

/// The words "Sign-in keys".
pub(crate) const SIGN_IN_KEYS: &str = "Sign-in keys";

/// The words "Clear".
pub(crate) const CLEAR: &str = "Clear";

/// The words "No account chosen".
pub(crate) const NO_ACCOUNT_CHOSEN: &str = "No account chosen";

/// The words "Choose an account to see its services and the apps that use it.".
pub(crate) const CHOOSE_ACCOUNT_HINT: &str =
    "Choose an account to see its services and the apps that use it.";

/// The words "Sign in to Your Accounts".
pub(crate) const SIGN_IN_TO_YOUR_ACCOUNTS: &str = "Sign in to Your Accounts";

/// The words "Nothing is signed in yet. Add an account and the apps you allow can use it.".
pub(crate) const NOTHING_SIGNED_IN: &str =
    "Nothing is signed in yet. Add an account and the apps you allow can use it.";

/// The words "Signed in".
pub(crate) const SIGNED_IN: &str = "Signed in";
