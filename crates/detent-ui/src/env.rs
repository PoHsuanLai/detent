//! What the window is started with: the schemas found, where the settings live, and where to
//! open. Provided as a context by whoever launches the app (`ds_blitz::AppConfig::with_context`),
//! so a test starts it on scratch directories.

use crate::inbox::Inbox;
use crate::inference::ReasonFeed;
use crate::lender::Lender;
use crate::live::BusSource;
use crate::marks::ProviderMarks;
use detent_files::{ConfigHome, DataHome, FirstRunMark, StateHome};
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
        }
    }

    /// This environment with the lending switch reaching `lender`.
    pub fn with_lender(self, lender: Lender) -> Env {
        Env { lender, ..self }
    }

    /// This environment with the picker's reason footer reading `reasons`.
    pub fn with_reasons(self, reasons: ReasonFeed) -> Env {
        Env { reasons, ..self }
    }
}
