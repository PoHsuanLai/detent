//! What Done writes: `ai.local_only` through the page's own write, the runtime through inferd's
//! picker module, then the mark that the first run happened.

use crate::env::Env;
use crate::files::{Edit, Files, apply};
use crate::inference::{is_menu, set};
use crate::live::{Daemon, open_once};
use detent_model::{AccountKey, LocalOnly, runtime_choice};

/// inferd's row that keeps AI on this computer.
const LOCAL_ONLY: &str = "ai.local_only";

/// Writes what the person chose. The first thing that fails is the answer: nothing after it runs,
/// and the mark is made only when everything before it was written.
pub(super) async fn save(
    env: &Env,
    local_only: LocalOnly,
    runtime: Option<AccountKey>,
) -> Result<(), String> {
    let entry = env
        .catalogue
        .entries()
        .map(|(_, entry)| entry)
        .find(|entry| entry.spec.path.0 == LOCAL_ONLY)
        .cloned()
        .ok_or_else(|| "inferd's settings schema is not installed.".to_owned())?;
    let edit = Edit {
        entry,
        value: toml::Value::String(local_only.word().to_owned()),
    };
    apply(env, &Files::new(), &edit).map_err(|error| error.to_string())?;
    if let Some(runtime) = runtime {
        choose(env, &runtime).await?;
    }
    env.first_run
        .record()
        .map_err(|error| format!("Could not remember this: {error}"))
}

/// Makes `runtime` the text slot's choice at inferd's picker.
async fn choose(env: &Env, runtime: &AccountKey) -> Result<(), String> {
    let (client, described) = open_once(&env.bus, &Daemon::inferd(), is_menu)
        .await
        .map_err(|absence| format!("The model could not be chosen: {absence}."))?;
    let (path, word) = runtime_choice(&described, runtime)
        .ok_or_else(|| format!("inferd lists no model of {} to choose.", runtime.0))?;
    set(&client, &path, &word).await
}
