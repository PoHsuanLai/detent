//! The window on scratch directories: real schemas seeded into a scratch `$XDG_DATA_HOME`, an
//! empty scratch `$XDG_CONFIG_HOME`, fixed desktop preferences. Nothing here reaches the real
//! config, the session bus or the portal.
#![allow(dead_code)]

pub mod drive;

use detent_files::ConfigHome;
use detent_testkit::{Scratch, seed_schemas};
use detent_ui::{App, Env, Opening};
use ds::prelude::{Scheme, SystemPrefs};
use ds_harness::{Harness, HarnessConfig, Viewport};
use ds_settings::SystemPrefsSource;

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
}

impl Sandbox {
    /// A world named `name` (unique per test) with the real schemas installed.
    pub fn new(name: &str) -> Sandbox {
        let scratch = Scratch::under(TMP, name);
        seed_schemas(&scratch.join("data"));
        Sandbox { scratch }
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
        )
    }

    /// A harness running the app in this world, light.
    pub fn harness(&self, opening: Opening) -> Harness {
        self.harness_in(opening, Scheme::Light)
    }

    /// A harness running the app in this world.
    pub fn harness_in(&self, opening: Opening, scheme: Scheme) -> Harness {
        Harness::new(App, self.config(opening, scheme))
    }

    /// The harness configuration that provides the app's environment.
    pub fn config(&self, opening: Opening, scheme: Scheme) -> HarnessConfig {
        HarnessConfig::new(VIEW).with_context(self.env(opening, scheme))
    }
}
