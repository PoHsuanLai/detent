//! Setting a row of inferd's settings module.

use ds_settings::live::LiveClient;
use ds_settings::schema::{KeyKind, KeyPath};

/// The rows whose value the picker reads: its menus.
pub(crate) fn is_menu(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Menu { .. })
}

/// Sets the row at `path` to `word`; the module answers when it has put it in force.
pub(crate) async fn set(client: &LiveClient, path: &KeyPath, word: &str) -> Result<(), String> {
    client
        .set(path, &toml::Value::String(word.to_owned()))
        .await
        .map_err(|error| error.to_string())
}
