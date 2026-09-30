//! What the window is started with: the schemas found, where the settings live, and where to
//! open. Provided as a context by whoever launches the app (`ds_blitz::AppConfig::with_context`),
//! so a test starts it on scratch directories.

use detent_files::ConfigHome;
use detent_model::{Catalogue, PageSlug, Warning, discover};
use ds_settings::SystemPrefsSource;
use std::path::PathBuf;
use std::sync::Arc;

/// Where the window opens.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Opening {
    /// The page to show first; the first page when `None` or unknown.
    pub page: Option<PageSlug>,
    /// What the search box holds; empty for none.
    pub query: String,
}

/// The app's ambient inputs.
#[derive(Debug, Clone)]
pub struct Env {
    /// Every key of every schema found.
    pub catalogue: Arc<Catalogue>,
    /// Schemas found and skipped.
    pub warnings: Vec<Warning>,
    /// The directory the settings files' paths are under.
    pub config_home: ConfigHome,
    /// Where the desktop's own preferences come from.
    pub system: SystemPrefsSource,
    /// Where the window opens.
    pub opening: Opening,
}

impl Env {
    /// The environment after scanning `schema_dirs` for schemas.
    pub fn discover(
        schema_dirs: &[PathBuf],
        config_home: ConfigHome,
        system: SystemPrefsSource,
        opening: Opening,
    ) -> Env {
        let found = discover(schema_dirs);
        Env {
            catalogue: Arc::new(Catalogue::build(&found.programs)),
            warnings: found.warnings,
            config_home,
            system,
            opening,
        }
    }
}
