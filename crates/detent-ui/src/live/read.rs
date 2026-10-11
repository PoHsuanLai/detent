//! Reading a module's rows and their values, as one reading even when the module moves meanwhile.

use super::io::absence_of;
use detent_model::{Absence, Described};
use ds_settings::live::{LiveClient, LiveError};
use ds_settings::schema::{KeyKind, KeyPath, KeySpec};
use std::collections::BTreeMap;

/// How many times a reading is made over when the module's rows move under it.
const ATTEMPTS: usize = 4;

/// What a reading asks of a module: its rows, and a row's value.
pub(crate) trait Rows {
    async fn describe(&self) -> Result<Vec<KeySpec>, LiveError>;
    async fn get(&self, path: &KeyPath) -> Result<toml::Value, LiveError>;
}

impl Rows for LiveClient {
    async fn describe(&self) -> Result<Vec<KeySpec>, LiveError> {
        Ok(LiveClient::describe(self).await?.key)
    }

    async fn get(&self, path: &KeyPath) -> Result<toml::Value, LiveError> {
        LiveClient::get(self, path).await
    }
}

/// The module's rows and the value of each row `valued` picks out. A row that was listed and is
/// unknown when asked for means the rows moved, so the whole reading is made again.
pub(crate) async fn read(
    module: &impl Rows,
    valued: fn(&KeyKind) -> bool,
) -> Result<Described, Absence> {
    for _ in 1..ATTEMPTS {
        match pass(module, valued).await {
            Err(LiveError::UnknownKey(_)) => {}
            done => return done.map_err(absence_of),
        }
    }
    pass(module, valued).await.map_err(absence_of)
}

async fn pass(module: &impl Rows, valued: fn(&KeyKind) -> bool) -> Result<Described, LiveError> {
    let keys = module.describe().await?;
    let mut values = BTreeMap::new();
    for spec in keys.iter().filter(|spec| valued(&spec.kind)) {
        if let Some(word) = module.get(&spec.path).await?.as_str() {
            values.insert(spec.path.clone(), word.to_owned());
        }
    }
    Ok(Described { keys, values })
}
