//! What the Sign in again row tells the person, by how the account signs in (the `sign_in` row).

use porter_core::SignInWay;

/// The way the `sign_in` row's slug names; none for a word this build does not know.
pub(super) fn sign_in_of(word: &str) -> Option<SignInWay> {
    toml::Value::String(word.to_owned()).try_into().ok()
}

/// The line under the Sign in again row. An account whose row is missing, or says it signs in
/// outside the desktop's accounts or not at all, gets the plain line.
pub fn reauth_help(way: Option<SignInWay>) -> &'static str {
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
