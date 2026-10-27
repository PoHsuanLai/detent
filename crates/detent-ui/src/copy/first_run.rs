//! The first-run window's words: how a local runtime stands.

use detent_model::{AccountStatus, Runtime};

/// What a person reads for how `runtime` stands: a runtime that is up is running.
pub(crate) fn runtime_state(runtime: &Runtime) -> String {
    match &runtime.status {
        AccountStatus::Ok => "Running".to_owned(),
        status => super::accounts::status_label(status),
    }
}

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
