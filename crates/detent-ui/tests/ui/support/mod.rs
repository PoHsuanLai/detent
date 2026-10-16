//! The window on scratch directories: real schemas seeded into a scratch `$XDG_DATA_HOME`, an
//! empty scratch `$XDG_CONFIG_HOME`, fixed desktop preferences. Nothing here reaches the real
//! config, the session bus or the portal.
#![allow(dead_code)]

pub mod accountd;
pub mod bus;
pub mod computers;
pub mod drive;
pub mod engines;
pub mod inferd;
pub mod lending;
pub mod memory;
pub mod spaces;
pub mod syncd;
pub mod updates;

use detent_files::{ConfigHome, DataHome, StateHome};
use detent_testkit::{Scratch, seed_schemas};
use detent_ui::{
    App, BusSource, Env, FirstRunApp, Lender, LogOut, Opening, Outbox, ReasonFeed, Updater, Vouch,
    inbox,
};
use ds::prelude::{Scheme, SystemPrefs};
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use ds_settings::SystemPrefsSource;
use ds_settings::live::AnyOwner;
use ds_settings::schema::Schema;
use lending::Units;
use porter_client::LendingConfig;
use updates::{Leaves, Service};

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
    units: Units,
    updates: Service,
    leaves: Leaves,
}

impl Sandbox {
    /// A world named `name` (unique per test) with the real schemas installed.
    pub fn new(name: &str) -> Sandbox {
        let scratch = Scratch::under(TMP, name);
        seed_schemas(&scratch.join("data"));
        let shipped = scratch.join("share/porter/inferd-tailnet.conf");
        std::fs::create_dir_all(shipped.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&shipped, lending::SHIPPED).expect("the shipped file");
        Sandbox {
            scratch,
            bus: BusSource::Absent,
            reasons: ReasonFeed::default(),
            units: Units::default(),
            updates: Service::absent(),
            leaves: Leaves::default(),
        }
    }

    /// This world with inferd looked for on the bus at `address`, whose stand-ins are every one
    /// taken for the service they play.
    pub fn on_bus(self, address: &str) -> Sandbox {
        self.on_bus_vouched(address, Vouch::of(AnyOwner))
    }

    /// This world with inferd looked for on the bus at `address`, the owner of each service's name
    /// judged by `vouch`.
    pub fn on_bus_vouched(self, address: &str, vouch: Vouch) -> Sandbox {
        Sandbox {
            bus: BusSource::Address(address.to_owned(), vouch),
            ..self
        }
    }

    /// This world with the reason footer reading `reasons`.
    pub fn with_reasons(self, reasons: ReasonFeed) -> Sandbox {
        Sandbox { reasons, ..self }
    }

    /// This world with the update service in this standing.
    pub fn with_service(self, updates: Service) -> Sandbox {
        Sandbox { updates, ..self }
    }

    /// The log-out the Software Update page starts, to read how often it was asked.
    pub fn leaves(&self) -> Leaves {
        self.leaves.clone()
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

    /// The scratch `$XDG_DATA_HOME`.
    pub fn data_home(&self) -> DataHome {
        DataHome(self.scratch.join("data"))
    }

    /// The scratch `$XDG_STATE_HOME`.
    pub fn state_home(&self) -> StateHome {
        StateHome(self.scratch.join("state"))
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

    /// Puts `text` at `relative` under the data home.
    pub fn write_data(&self, relative: &str, text: &str) {
        let path = self.data_home().0.join(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    }

    /// The app's environment, the desktop reporting `scheme`.
    pub fn env(&self, opening: Opening, scheme: Scheme) -> Env {
        Env::discover(
            &[self.scratch.join("data/quire/settings")],
            self.config_home(),
            self.data_home(),
            &self.state_home(),
            SystemPrefsSource::Fixed(SystemPrefs::default().with_scheme(scheme)),
            opening,
            self.bus.clone(),
        )
        .with_reasons(self.reasons.clone())
        .with_lender(self.lender())
        .with_updates(Updater::over(self.updates.clone()))
        .with_log_out(LogOut::over(self.leaves.clone()))
        .with_os_release(self.os_release_path())
    }

    /// Where this world's `os-release` is: a scratch file, never the computer's own.
    pub fn os_release_path(&self) -> std::path::PathBuf {
        self.scratch.join("etc/os-release")
    }

    /// This world's computer is the distro `text` describes (an `os-release` file's contents).
    pub fn write_os_release(&self, text: &str) {
        let path = self.os_release_path();
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    }

    /// The lending switch over the scratch folders and the recording manager: never the real
    /// systemd and never the real home.
    pub fn lender(&self) -> Lender {
        Lender::over(
            LendingConfig::new(
                self.scratch.join("share/porter/inferd-tailnet.conf"),
                self.scratch.join("config/systemd/user"),
            ),
            self.units.clone(),
        )
    }

    /// The service manager the lending switch talks to, to read its calls or make it refuse.
    pub fn units(&self) -> Units {
        self.units.clone()
    }

    /// The drop-in the switch wrote, if it is there.
    pub fn drop_in(&self) -> Option<String> {
        self.read("systemd/user/inferd.service.d/tailnet.conf")
    }

    /// A harness running the app in this world, light.
    pub fn harness(&self, opening: Opening) -> Harness {
        self.harness_in(opening, Scheme::Light)
    }

    /// A harness running the app in this world.
    pub fn harness_in(&self, opening: Opening, scheme: Scheme) -> Harness {
        Harness::new(App, self.config(opening, scheme))
    }

    /// A harness running the first-run window in this world.
    pub fn first_run_harness(&self) -> Harness {
        self.first_run_harness_in(Scheme::Light)
    }

    /// The same, the desktop reporting `scheme`.
    pub fn first_run_harness_in(&self, scheme: Scheme) -> Harness {
        Harness::new(FirstRunApp, self.config(Opening::default(), scheme))
    }

    /// Takes a program's installed schema away, as if its package were not installed.
    pub fn remove_schema(&self, name: &str) {
        let path = self
            .scratch
            .join("data/quire/settings")
            .join(format!("{name}.settings.toml"));
        std::fs::remove_file(path).expect("an installed schema");
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
