//! The values of one settings file.

use detent_model::ConfigFile;
use ds_settings::schema::KeyPath;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// `$XDG_CONFIG_HOME` (or a scratch directory standing in for it): the directory the settings
/// files' `<app>/<name>.toml` paths are under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigHome(pub PathBuf);

impl ConfigHome {
    /// Where `file` is on disk.
    pub fn path_of(&self, file: &ConfigFile) -> PathBuf {
        self.0.join(file.relative())
    }
}

/// The current value of every key of a file that has one: what the file says where it says
/// something valid, and the key's default elsewhere.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileValues {
    values: BTreeMap<KeyPath, toml::Value>,
    /// The file's top-level `version`, carried through a save when the file has one.
    pub(crate) version: Option<toml::Value>,
}

impl FileValues {
    /// The value of `path`.
    pub fn get(&self, path: &KeyPath) -> Option<&toml::Value> {
        self.values.get(path)
    }

    /// These values with `path` set to `value`.
    #[must_use]
    pub fn with(mut self, path: &KeyPath, value: toml::Value) -> FileValues {
        self.values.insert(path.clone(), value);
        self
    }

    pub(crate) fn from_parts(
        values: BTreeMap<KeyPath, toml::Value>,
        version: Option<toml::Value>,
    ) -> FileValues {
        FileValues { values, version }
    }
}
