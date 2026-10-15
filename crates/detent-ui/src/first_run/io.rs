//! What Done writes: `ai.local_only` through the page's own write, then the mark that the first
//! run happened. The assistant's model is written as it is chosen, by the picker.

use crate::env::Env;
use crate::files::{Edit, Files, apply};
use detent_model::LocalOnly;

/// inferd's row that keeps AI on this computer.
const LOCAL_ONLY: &str = "ai.local_only";

/// Writes what the person chose. The first thing that fails is the answer: nothing after it runs,
/// and the mark is made only when everything before it was written.
pub(super) fn save(env: &Env, local_only: LocalOnly) -> Result<(), String> {
    let entry = env
        .catalogue
        .entries()
        .map(|(_, entry)| entry)
        .find(|entry| entry.spec.path.0 == LOCAL_ONLY)
        .cloned()
        .ok_or_else(|| "The model settings are not installed.".to_owned())?;
    let edit = Edit {
        entry,
        value: toml::Value::String(local_only.word().to_owned()),
    };
    apply(env, &Files::new(), &edit).map_err(|error| error.to_string())?;
    env.first_run
        .record()
        .map_err(|error| format!("Could not remember this: {error}"))
}
