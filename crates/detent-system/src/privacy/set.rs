//! The calls the page makes on the compositor's module.

use crate::bus::plain;
use crate::modules::Module;
use detent_model::PrivacySwitch;
use ds_settings::schema::KeyPath;

/// Allows or denies at `path`; the module's own words when it refuses.
pub async fn set(client: &Module, path: &KeyPath, to: PrivacySwitch) -> Result<(), String> {
    client
        .set(path, &toml::Value::String(to.word().to_owned()))
        .await
        .map_err(|error| plain(&error))
}

/// Asks again at `path`; the module's own words when it refuses.
pub async fn forget(client: &Module, path: &KeyPath) -> Result<(), String> {
    client.invoke(path).await.map_err(|error| plain(&error))
}
