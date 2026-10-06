//! The window on scratch directories: real schemas seeded into a scratch `$XDG_DATA_HOME`, an
//! empty scratch `$XDG_CONFIG_HOME`, fixed desktop preferences. Nothing here reaches the real
//! config, the session bus or the portal.
#![allow(dead_code)]

pub mod bus;
pub mod drive;
pub mod inferd;

use detent_files::ConfigHome;
use detent_testkit::{Scratch, seed_schemas};
use detent_ui::{App, BusSource, Env, Opening, Outbox, ReasonFeed, inbox};
use ds::prelude::{Scheme, SystemPrefs};
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use ds_settings::SystemPrefsSource;
use ds_settings::schema::Schema;

pub const TMP: &str = env!("CARGO_TARGET_TMPDIR");

/// The window's size in the app and in the pictures.
pub const VIEW: Viewport = Viewport {
    width: 900,
    height: 640,
    scale_percent: 100,
};

/// One test's scratch world.
pub struct Sandbox {
    scratch: Scratch,
    bus: BusSource,
    reasons: ReasonFeed,
}

impl Sandbox {
    /// A world named `name` (unique per test) with the real schemas installed.
    pub fn new(name: &str) -> Sandbox {
        let scratch = Scratch::under(TMP, name);
        seed_schemas(&scratch.join("data"));
        Sandbox {
            scratch,
            bus: BusSource::Absent,
            reasons: ReasonFeed::default(),
        }
    }

    /// This world with inferd looked for on the bus at `address`.
    pub fn on_bus(self, address: &str) -> Sandbox {
        Sandbox {
            bus: BusSource::Address(address.to_owned()),
            ..self
        }
    }

    /// This world with the reason footer reading `reasons`.
    pub fn with_reasons(self, reasons: ReasonFeed) -> Sandbox {
        Sandbox { reasons, ..self }
    }

    /// Installs one more program's schema as text, as a program would under `$XDG_DATA_DIRS`.
    pub fn add_schema(&self, name: &str, text: &str) {
        let path = self
            .scratch
            .join("data/quire/settings")
            .join(format!("{name}.settings.toml"));
        std::fs::write(path, text).expect("schema");
    }

    /// The scratch `$XDG_CONFIG_HOME`.
    pub fn config_home(&self) -> ConfigHome {
        ConfigHome(self.scratch.join("config"))
    }

    /// The text of a settings file under the config home, if it was written.
    pub fn read(&self, relative: &str) -> Option<String> {
        std::fs::read_to_string(self.config_home().0.join(relative)).ok()
    }

    /// Puts `text` at `relative` under the config home.
    pub fn write(&self, relative: &str, text: &str) {
        let path = self.config_home().0.join(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    }

    /// The app's environment, the desktop reporting `scheme`.
    pub fn env(&self, opening: Opening, scheme: Scheme) -> Env {
        Env::discover(
            &[self.scratch.join("data/quire/settings")],
            self.config_home(),
            SystemPrefsSource::Fixed(SystemPrefs {
                scheme,
                ..SystemPrefs::default()
            }),
            opening,
            self.bus.clone(),
        )
        .with_reasons(self.reasons.clone())
    }

    /// A harness running the app in this world, light.
    pub fn harness(&self, opening: Opening) -> Harness {
        self.harness_in(opening, Scheme::Light)
    }

    /// A harness running the app in this world.
    pub fn harness_in(&self, opening: Opening, scheme: Scheme) -> Harness {
        Harness::new(App, self.config(opening, scheme))
    }

    /// Installs `schema` beside the real ones, as a program's package would.
    pub fn install_schema(&self, schema: &Schema) {
        schema
            .write_to(&self.scratch.join("data/quire/settings"))
            .expect("write the schema");
    }

    /// A harness running the app in this world, and the way to tell its window things.
    pub fn harness_told(&self, opening: Opening) -> (Harness, Outbox) {
        let (outbox, inbox) = inbox();
        let env = Env {
            inbox,
            ..self.env(opening, Scheme::Light)
        };
        let config = HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_context(env);
        (Harness::new(App, config), outbox)
    }

    /// The harness configuration that provides the app's environment. Timers run on the
    /// harness's virtual clock, so what a test sees depends on what it did and never on how
    /// loaded the host is.
    pub fn config(&self, opening: Opening, scheme: Scheme) -> HarnessConfig {
        HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_context(self.env(opening, scheme))
    }
}
