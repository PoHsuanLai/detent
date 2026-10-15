//! Setting a row of inferd's settings module.

use ds_settings::live::LiveClient;
use ds_settings::schema::{KeyKind, KeyPath};

/// The rows whose value the picker reads: its menus, and the lists of choices from a menu (the
/// models Automatic chooses among).
pub(crate) fn is_choice(kind: &KeyKind) -> bool {
    match kind {
        KeyKind::Menu { .. } => true,
        KeyKind::List(inner) => matches!(**inner, KeyKind::Menu { .. }),
        _ => false,
    }
}

/// Sets the row at `path` to `word`; the module answers when it has put it in force.
pub(crate) async fn set(client: &LiveClient, path: &KeyPath, word: &str) -> Result<(), String> {
    client
        .set(path, &toml::Value::String(word.to_owned()))
        .await
        .map_err(|error| error.to_string())
}

/// Sets the pool at `path` to `words`; the module refuses a model it does not know.
pub(crate) async fn set_pool(
    client: &LiveClient,
    path: &KeyPath,
    words: &[String],
) -> Result<(), String> {
    let list = words.iter().cloned().map(toml::Value::String).collect();
    client
        .set(path, &toml::Value::Array(list))
        .await
        .map_err(|error| error.to_string())
}
