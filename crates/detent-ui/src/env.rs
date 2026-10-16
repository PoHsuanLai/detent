//! What the window is started with: the schemas found, where the settings live, and where to
//! open. Provided as a context by whoever launches the app (`ds_blitz::AppConfig::with_context`),
//! so a test starts it on scratch directories.

use crate::engine_install::EngineDesk;
use crate::inbox::Inbox;
use crate::inference::ReasonFeed;
use crate::lender::Lender;
use crate::live::BusSource;
use crate::marks::ProviderMarks;
use crate::updates::{LogOut, Updater};
use detent_files::{ConfigHome, DataHome, FirstRunMark, KeymapFiles, StateHome};
use detent_model::{Catalogue, PageSlug, Warning, discover};
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

/// The app's ambient inputs.
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
    /// Where inferd is looked for.
    pub bus: BusSource,
    /// Where the model picker's reason footer reads inferd's `Why` and `Declined` from.
    pub reasons: ReasonFeed,
    /// The marks of the providers the Accounts page shows.
    pub marks: ProviderMarks,
    /// How the switch that lends this computer's models reaches the files and the service
    /// manager; the person's own session unless a test hands in another.
    pub lender: Lender,
    /// The keymap files the Shortcuts list reads, and the one it writes: the person's own.
    pub keymap: KeymapFiles,
    /// The update service the Software Update page follows, looked for on `bus`.
    pub updates: Updater,
    /// How the Software Update page logs the person out once an update is installed.
    pub log_out: LogOut,
    /// The file that says which distro this is (`os-release`), read to word an install line.
    pub os_release: PathBuf,
    /// The folders of desktop entries, in the order of precedence: where the Privacy page finds
    /// the name and icon of a program. None until the launcher names them.
    pub application_dirs: Vec<PathBuf>,
    /// What the sheet that downloads an engine does to the computer: the person's own unless a
    /// test hands in another.
    pub engines: EngineDesk,
}

impl Env {
    /// The environment after scanning `schema_dirs` for schemas.
    pub fn discover(
        schema_dirs: &[PathBuf],
        config_home: ConfigHome,
        data_home: DataHome,
        state_home: &StateHome,
        system: SystemPrefsSource,
        opening: Opening,
        bus: BusSource,
    ) -> Env {
        let found = discover(schema_dirs);
        let keymap = KeymapFiles::under(&config_home);
        let updates = Updater::on(bus.clone());
        let engines = EngineDesk::system(bus.clone(), &data_home.0);
        Env {
            catalogue: Arc::new(Catalogue::build(&found.programs)),
            warnings: found.warnings,
            config_home,
            data_home,
            first_run: FirstRunMark::under(state_home),
            system,
            opening,
            inbox: Inbox::default(),
            bus,
            reasons: ReasonFeed::default(),
            marks: ProviderMarks::shipped(),
            lender: Lender::from_env(),
            keymap,
            updates,
            log_out: LogOut::power_menu(),
            os_release: Environment::system().os_release,
            application_dirs: Vec::new(),
            engines,
        }
    }

    /// This environment with the lending switch reaching `lender`.
    pub fn with_lender(self, lender: Lender) -> Env {
        Env { lender, ..self }
    }

    /// This environment with the Shortcuts list over `keymap`.
    pub fn with_keymap(self, keymap: KeymapFiles) -> Env {
        Env { keymap, ..self }
    }

    /// This environment with the Software Update page following `updates`.
    pub fn with_updates(self, updates: Updater) -> Env {
        Env { updates, ..self }
    }

    /// This environment with the Software Update page logging out through `log_out`.
    pub fn with_log_out(self, log_out: LogOut) -> Env {
        Env { log_out, ..self }
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

    /// This environment with the engine sheet working through `engines`.
    pub fn with_engines(self, engines: EngineDesk) -> Env {
        Env { engines, ..self }
    }

    /// This environment with the picker's reason footer reading `reasons`.
    pub fn with_reasons(self, reasons: ReasonFeed) -> Env {
        Env { reasons, ..self }
    }
}
