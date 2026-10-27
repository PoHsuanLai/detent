//! The handle a pane keeps of a module while it is there: the calls the panes make on it, and no
//! more. The bus's handle is the module's `LiveClient`; a fake's is a module in the process.

use crate::bus::{Rows, read};
use detent_model::{Absence, Described, Work};
use ds_settings::live::{LiveClient, LiveError, LiveSchema};
use ds_settings::schema::{KeyKind, KeyPath, KeySpec};
use std::fmt::Debug;
use std::sync::Arc;

/// What a pane asks of a module it follows.
pub trait ModuleHandle: Debug + Send + Sync {
    /// The module's rows now.
    fn describe(&self) -> Work<'_, Result<LiveSchema, LiveError>>;
    /// A row's value.
    fn get<'a>(&'a self, key: &'a KeyPath) -> Work<'a, Result<toml::Value, LiveError>>;
    /// Sets a row, or runs it when it is an action row.
    fn set<'a>(
        &'a self,
        key: &'a KeyPath,
        value: &'a toml::Value,
    ) -> Work<'a, Result<(), LiveError>>;
}

impl ModuleHandle for LiveClient {
    fn describe(&self) -> Work<'_, Result<LiveSchema, LiveError>> {
        Box::pin(LiveClient::describe(self))
    }

    fn get<'a>(&'a self, key: &'a KeyPath) -> Work<'a, Result<toml::Value, LiveError>> {
        Box::pin(LiveClient::get(self, key))
    }

    fn set<'a>(
        &'a self,
        key: &'a KeyPath,
        value: &'a toml::Value,
    ) -> Work<'a, Result<(), LiveError>> {
        Box::pin(LiveClient::set(self, key, value))
    }
}

/// A module that is there, cheap to clone.
#[derive(Debug, Clone)]
pub struct Module(Arc<dyn ModuleHandle>);

impl Module {
    /// The module `handle` reaches.
    pub fn new(handle: Arc<dyn ModuleHandle>) -> Module {
        Module(handle)
    }

    /// The module's rows now.
    pub async fn describe(&self) -> Result<LiveSchema, LiveError> {
        self.0.describe().await
    }

    /// A row's value.
    pub async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        self.0.get(key).await
    }

    /// Sets a row.
    pub async fn set(&self, key: &KeyPath, value: &toml::Value) -> Result<(), LiveError> {
        self.0.set(key, value).await
    }

    /// Runs an action row: a `set` of `true`.
    pub async fn invoke(&self, key: &KeyPath) -> Result<(), LiveError> {
        self.set(key, &toml::Value::Boolean(true)).await
    }
}

impl Rows for Module {
    async fn describe(&self) -> Result<Vec<KeySpec>, LiveError> {
        Ok(Module::describe(self).await?.key)
    }

    async fn get(&self, path: &KeyPath) -> Result<toml::Value, LiveError> {
        Module::get(self, path).await
    }
}

/// The rows of `module` as they stand now, a value for each row `valued` picks out.
pub async fn reread(module: &Module, valued: fn(&KeyKind) -> bool) -> Result<Described, Absence> {
    read(module, valued).await
}
