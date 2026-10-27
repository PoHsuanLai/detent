//! The first-run window's words: how a local runtime stands.

use detent_model::{AccountStatus, Runtime};

/// What a person reads for how `runtime` stands: a runtime that is up is running.
pub(crate) fn runtime_state(runtime: &Runtime) -> String {
    match &runtime.status {
        AccountStatus::Ok => "Running".to_owned(),
        status => super::accounts::status_label(status),
    }
}

/// The words "The model settings are not installed.".
pub(crate) const THE_MODEL_SETTINGS_ARE_NOT_INSTALLED: &str =
    "The model settings are not installed.";

/// The words "Could not remember this: {error}".
pub(crate) fn could_not_remember_this(error: impl std::fmt::Display) -> String {
    format!("Could not remember this: {error}")
}

/// The words "Welcome".
pub(crate) const WELCOME: &str = "Welcome";

/// The words "Keep AI on this computer".
pub(crate) const KEEP_AI_ON_THIS_COMPUTER: &str = "Keep AI on this computer";

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::AccountKey;

    #[test]
    fn a_runtime_that_is_up_is_running() {
        let runtime = Runtime {
            key: AccountKey("ollama".to_owned()),
            label: "Ollama".to_owned(),
            status: AccountStatus::Ok,
        };
        assert_eq!(runtime_state(&runtime), "Running");
    }
}
