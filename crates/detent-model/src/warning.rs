//! Why a schema was left out.

use ds_settings::schema::AppId;
use std::fmt;

/// What made a schema stale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleReason {
    /// The schema says a version this app does not read.
    UnsupportedVersion(u16),
    /// Its `file` is not `<app-dir>/<name>.toml` under the config home.
    UnsafeFile(String),
}

/// A schema that was found and skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// Whose schema.
    pub app: AppId,
    /// Why it was skipped.
    pub reason: StaleReason,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let app = &self.app.0;
        match &self.reason {
            StaleReason::UnsupportedVersion(version) => {
                write!(
                    f,
                    "skipping stale schema for {app}: version {version} is not read"
                )
            }
            StaleReason::UnsafeFile(file) => {
                write!(
                    f,
                    "skipping stale schema for {app}: file `{file}` is not <app>/<name>.toml"
                )
            }
        }
    }
}
