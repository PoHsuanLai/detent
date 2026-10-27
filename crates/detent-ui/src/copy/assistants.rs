//! The assistants pane's words.

/// The menu line for "the agent's own choice": no model is written.
pub(crate) fn agents_own() -> &'static str {
    "The agent\u{2019}s own choice"
}

/// The words "Add a coding assistant".
pub(crate) const ADD_A_CODING_ASSISTANT: &str = "Add a coding assistant";

/// The words "Coding assistants".
pub(crate) const CODING_ASSISTANTS: &str = "Coding assistants";

/// The words "Looking for coding assistants".
pub(crate) const LOOKING_FOR_CODING_ASSISTANTS: &str = "Looking for coding assistants";

/// The words "Could not save that choice. {why}".
pub(crate) fn could_not_save_that_choice(why: impl std::fmt::Display) -> String {
    format!("Could not save that choice. {why}")
}

/// The words "Sign in".
pub(crate) const SIGN_IN: &str = "Sign in";

/// The words "Remove {}".
pub(crate) fn remove(a: impl std::fmt::Display) -> String {
    format!("Remove {a}")
}

/// The words "Sign in with".
pub(crate) const SIGN_IN_WITH: &str = "Sign in with";

/// The words "Automatic".
pub(crate) const AUTOMATIC: &str = "Automatic";

/// The words "Advanced".
pub(crate) const ADVANCED: &str = "Advanced";

/// The words "Advanced, {}".
pub(crate) fn advanced(a: impl std::fmt::Display) -> String {
    format!("Advanced, {a}")
}

/// The words "Ask me each time".
pub(crate) const ASK_ME_EACH_TIME: &str = "Ask me each time";

/// The words "The assistant asks before it uses this account.".
pub(crate) const THE_ASSISTANT_ASKS_BEFORE_IT_USES: &str =
    "The assistant asks before it uses this account.";

/// The words "Signed-in accounts that offer AI appear here. Each starts off.".
pub(crate) const SIGNED_IN_ACCOUNTS_THAT_OFFER_AI: &str =
    "Signed-in accounts that offer AI appear here. Each starts off.";

/// The words "No models right now".
pub(crate) const NO_MODELS_RIGHT_NOW: &str = "No models right now";

/// The words "Ready".
pub(crate) const READY: &str = "Ready";

/// The words "Not answering".
pub(crate) const NOT_ANSWERING: &str = "Not answering";

/// The words "Could not save where the assistant may run. {why}".
pub(crate) fn could_not_save_where_the_assistant(why: impl std::fmt::Display) -> String {
    format!("Could not save where the assistant may run. {why}")
}

/// The words "This computer".
pub(crate) const THIS_COMPUTER: &str = "This computer";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_agents_own_choice_is_named() {
        assert_eq!(agents_own(), "The agent\u{2019}s own choice");
    }
}
