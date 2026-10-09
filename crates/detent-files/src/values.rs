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

/// `$XDG_DATA_HOME` (or a scratch directory standing in for it): where docket keeps the agents it
/// installed and what each last offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataHome(pub PathBuf);

/// The current value of every key of a file that has one: what the file says where the key's
/// control can hold it, and the key's default elsewhere.
///
/// A value the control cannot hold (a gesture bound to a key chord, an out-of-range number, an
/// unknown word) is kept apart as it was written: the window shows it read-only, and nothing
/// ever rewrites it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileValues {
    values: BTreeMap<KeyPath, toml::Value>,
    as_written: BTreeMap<KeyPath, toml::Value>,
}

impl FileValues {
    /// The value of `path`.
    pub fn get(&self, path: &KeyPath) -> Option<&toml::Value> {
        self.values.get(path)
    }

    /// The value the file holds at `path` when no control can represent it.
    pub fn as_written(&self, path: &KeyPath) -> Option<&toml::Value> {
        self.as_written.get(path)
    }

    /// These values with `path` set to `value`, which the person chose, so it is representable.
    #[must_use]
    pub fn with(mut self, path: &KeyPath, value: toml::Value) -> FileValues {
        self.as_written.remove(path);
        self.values.insert(path.clone(), value);
        self
    }

    pub(crate) fn from_parts(
        values: BTreeMap<KeyPath, toml::Value>,
        as_written: BTreeMap<KeyPath, toml::Value>,
    ) -> FileValues {
        FileValues { values, as_written }
    }
}
