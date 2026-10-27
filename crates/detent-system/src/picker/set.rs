//! Setting a row of inferd's settings module.

use crate::bus::plain;
use crate::modules::Module;
use ds_settings::schema::KeyPath;

/// Sets the row at `path` to `word`; the module answers when it has put it in force.
pub async fn set(client: &Module, path: &KeyPath, word: &str) -> Result<(), String> {
    client
        .set(path, &toml::Value::String(word.to_owned()))
        .await
        .map_err(|error| plain(&error))
}

/// Sets the pool at `path` to `words`; the module refuses a model it does not know.
pub async fn set_pool(client: &Module, path: &KeyPath, words: &[String]) -> Result<(), String> {
    let list = words.iter().cloned().map(toml::Value::String).collect();
    client
        .set(path, &toml::Value::Array(list))
        .await
        .map_err(|error| plain(&error))
}
