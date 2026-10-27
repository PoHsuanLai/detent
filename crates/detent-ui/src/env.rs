//! What the window is started with: the schemas found, where the settings live, and where to
//! open. Provided as a context by whoever launches the app (`ds_blitz::AppConfig::with_context`),
//! so a test starts it on scratch directories.

use crate::marks::ProviderMarks;
use detent_client::{
    Client, ConfigHome, DataHome, ExtensionHome, FirstRunMark, KeymapFiles, StateHome,
};
use detent_model::{Catalogue, Inbox, OriginRule, PageSlug, Warning, discover};
use ds_helpers::Environment;
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

/// The app's ambient inputs: the one door to the computer (`client`), and what only the window
/// holds.
#[derive(Debug, Clone)]
pub struct Env {
    /// Every key of every schema found.
    pub catalogue: Arc<Catalogue>,
    /// Schemas found and skipped.
    pub warnings: Vec<Warning>,
    /// The directory the settings files' paths are under.
    pub config_home: ConfigHome,
    /// Where docket keeps the agents it installed and what each offered.
    pub data_home: DataHome,
    /// Whether the first run happened, kept under the state home.
    pub first_run: FirstRunMark,
    /// Where the desktop's own preferences come from.
    pub system: SystemPrefsSource,
    /// Where the window opens.
    pub opening: Opening,
    /// What the window is told while it runs; empty until the launcher gives it one.
    pub inbox: Inbox,
    /// Every service the window reaches.
    pub client: Client,
    /// The marks of the providers the Accounts page shows.
    pub marks: ProviderMarks,
    /// The keymap files the Shortcuts list reads, and the one it writes: the person's own.
    pub keymap: KeymapFiles,
    /// The file that says which distro this is (`os-release`), read to word an install line.
    pub os_release: PathBuf,
    /// The folders of desktop entries, in the order of precedence: where the Privacy page finds
    /// the name and icon of a program. None until the launcher names them.
    pub application_dirs: Vec<PathBuf>,
    /// How those folders are judged: only an entry in one closed to everyone but an
    /// administrator may name a program. A test declares its scratch folder to be one.
    pub entry_rule: OriginRule,
    /// The Extensions installed, which the Look menu offers and the Extensions page lists; the
    /// person's own folders only unless the launcher names the system's.
    pub extensions: ExtensionHome,
    /// The system's data directories, in the order of precedence: where the Extensions and the
    /// apps' published style parts are looked for besides the person's own. None until the
    /// launcher names them, so a test reads only its scratch folders.
    pub system_data_dirs: Vec<PathBuf>,
}

/// The person's own folders the window reads and writes: settings, data and state (or scratch
/// folders standing in for them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Homes {
    /// Where the settings files are.
    pub config: ConfigHome,
    /// Where docket's agents and the added pictures are.
    pub data: DataHome,
    /// Where the first-run mark is.
    pub state: StateHome,
}

impl Env {
    /// The environment after scanning `schema_dirs` for schemas, reaching the computer through
    /// `client`.
    pub fn discover(
        schema_dirs: &[PathBuf],
        homes: Homes,
        system: SystemPrefsSource,
        opening: Opening,
        client: Client,
    ) -> Env {
        let Homes {
            config: config_home,
            data: data_home,
            state: state_home,
        } = homes;
        let found = discover(schema_dirs);
        let keymap = KeymapFiles::under(&config_home);
        let extensions = ExtensionHome::under(&config_home, &data_home, &[]);
        Env {
            catalogue: Arc::new(Catalogue::build(&found.programs)),
            warnings: found.warnings,
            config_home,
            data_home,
            first_run: FirstRunMark::under(&state_home),
            system,
            opening,
            inbox: Inbox::default(),
            client,
            marks: ProviderMarks::shipped(),
            keymap,
            os_release: Environment::system().os_release,
            application_dirs: Vec::new(),
            entry_rule: OriginRule::Ownership,
            extensions,
            system_data_dirs: Vec::new(),
        }
    }

    /// This environment with the Shortcuts list over `keymap`.
    pub fn with_keymap(self, keymap: KeymapFiles) -> Env {
        Env { keymap, ..self }
    }

    /// This environment reading the distro from `os_release`.
    pub fn with_os_release(self, os_release: PathBuf) -> Env {
        Env { os_release, ..self }
    }

    /// This environment finding the programs' desktop entries in `application_dirs`.
    pub fn with_application_dirs(self, application_dirs: Vec<PathBuf>) -> Env {
        Env {
            application_dirs,
            ..self
        }
    }

    /// This environment judging the desktop entries' folders by `entry_rule`.
    pub fn with_entry_rule(self, entry_rule: OriginRule) -> Env {
        Env { entry_rule, ..self }
    }

    /// This environment looking for Extensions and published style parts in `system_data_dirs`
    /// as well as the person's own data home.
    pub fn with_system_data_dirs(self, system_data_dirs: Vec<PathBuf>) -> Env {
        let extensions =
            ExtensionHome::under(&self.config_home, &self.data_home, &system_data_dirs);
        Env {
            extensions,
            system_data_dirs,
            ..self
        }
    }
}
