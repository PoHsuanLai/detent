//! The calls the page makes on the compositor's module.

use crate::live::plain;
use detent_model::PrivacySwitch;
use ds_settings::live::LiveClient;
use ds_settings::schema::{KeyKind, KeyPath};

/// What the person reads when the compositor is gone as a change is sent.
pub(super) const NOT_AVAILABLE: &str =
    "Privacy isn\u{2019}t available right now. Try again in a moment.";

/// The rows whose value is read with the rows: the switches and the administrator's read-outs.
pub(super) fn is_valued(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Toggle { .. } | KeyKind::Fixed { .. })
}

/// Allows or denies at `path`; the module's own words when it refuses.
pub(super) async fn set(
    client: &LiveClient,
    path: &KeyPath,
    to: PrivacySwitch,
) -> Result<(), String> {
    client
        .set(path, &toml::Value::String(to.word().to_owned()))
        .await
        .map_err(|error| plain(&error))
}

/// Asks again at `path`; the module's own words when it refuses.
pub(super) async fn forget(client: &LiveClient, path: &KeyPath) -> Result<(), String> {
    client.invoke(path).await.map_err(|error| plain(&error))
}
