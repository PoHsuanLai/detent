//! Building a [`Client`]: over the real services of the person's session, with any of them
//! replaced by what a test hands in.

use crate::client::Client;
use crate::picker::{AgentSearch, ReasonFeed};
use detent_files::{CacheHome, ConfigHome, DataHome, PictureDirs};
use detent_system::accounts::{AccountsLink, OnBus as AccountsOnBus};
use detent_system::bluetooth::{BluetoothLink, OnBluetooth};
use detent_system::bus::BusSource;
use detent_system::chooser::{ChooserLink, OnPortal};
use detent_system::computers::{ComputersLink, OnBus as ComputersOnBus};
use detent_system::displays::{DisplaysLink, OnCompositor};
use detent_system::engine_install::{Desk, OnEngines};
use detent_system::engines::{EnginesLink, OnBus as EnginesOnBus};
use detent_system::hot_corners::{CornerDesk, OnCorners};
use detent_system::lending::Lender;
use detent_system::memory_consent::{MemoryLink, OnBus as MemoryOnBus};
use detent_system::modules::{ModulesLink, OnBus as ModulesOnBus};
use detent_system::picker::{OnClient as PickerOnClient, PickerLink};
use detent_system::privacy::{OnClient as PrivacyOnClient, PrivacyLink};
use detent_system::running::{OnBus as RunningOnBus, RunningLink};
use detent_system::screen_sharing::{OnBus as ScreenSharingOnBus, ScreenSharingLink};
use detent_system::spaces::{OnBus as SpacesOnBus, SpacesLink};
use detent_system::sync::{OnBus as SyncOnBus, SyncLink};
use detent_system::updates::{Departure, OnBus, PowerMenu, UpdatesLink};
use detent_system::wallpaper::{OnDisk, Pictures, Thumbnails};
use detent_system::wifi::{OnWifi, WifiLink};
use std::path::PathBuf;
use std::sync::Arc;

/// The person's own folders the client reads and writes: settings, data and cache (or scratch
/// folders standing in for them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientHomes {
    /// Where the settings files are.
    pub config: ConfigHome,
    /// Where docket's agents and the added pictures are.
    pub data: DataHome,
    /// Where the Wallpaper page's small pictures are kept.
    pub cache: CacheHome,
}

/// A client being built: every service is the person's own until a setter says otherwise.
#[derive(Debug, Clone)]
pub struct ClientBuilder(Client);

impl ClientBuilder {
    /// A client over the real services, looked for on `bus`, with its files under `homes`.
    pub fn over(bus: BusSource, homes: ClientHomes) -> ClientBuilder {
        let ClientHomes {
            config,
            data,
            cache,
        } = homes;
        let chooser: Arc<dyn ChooserLink> = Arc::new(OnPortal(bus.clone()));
        let desk = Arc::new(OnEngines::new(bus.clone(), Arc::clone(&chooser), &data.0));
        let pictures = Arc::new(OnDisk::new(
            Arc::clone(&chooser),
            PictureDirs::standard(&data),
            Thumbnails::under(&cache),
        ));
        let corners = Arc::new(OnCorners::new(Arc::clone(&chooser), Vec::new()));
        ClientBuilder(Client {
            chooser,
            modules: Arc::new(ModulesOnBus(bus.clone())),
            accounts: Arc::new(AccountsOnBus(bus.clone())),
            sync: Arc::new(SyncOnBus(bus.clone())),
            picker: Arc::new(PickerOnClient),
            privacy: Arc::new(PrivacyOnClient),
            screen_sharing: Arc::new(ScreenSharingOnBus(bus.clone())),
            spaces: Arc::new(SpacesOnBus(bus.clone())),
            memory: Arc::new(MemoryOnBus(bus.clone())),
            running: Arc::new(RunningOnBus(bus.clone())),
            computers: Arc::new(ComputersOnBus(bus.clone())),
            engines: Arc::new(EnginesOnBus(bus.clone())),
            reasons: ReasonFeed::default(),
            lender: Lender::from_env(),
            updates: Arc::new(OnBus(bus.clone())),
            log_out: Arc::new(PowerMenu),
            wifi: Arc::new(OnWifi(bus.clone())),
            bluetooth: Arc::new(OnBluetooth::new(bus.clone())),
            displays: Arc::new(OnCompositor(bus.clone())),
            desk,
            pictures,
            corners,
            search: AgentSearch {
                config_home: config,
                data_home: data,
                programs_path: String::new(),
            },
            agents_program: None,
            bus,
        })
    }

    /// The client built.
    pub fn build(self) -> Client {
        self.0
    }

    /// With the lending switch reaching `lender`.
    #[must_use]
    pub fn with_lender(self, lender: Lender) -> ClientBuilder {
        ClientBuilder(Client { lender, ..self.0 })
    }

    /// With the Software Update pane following `updates`.
    #[must_use]
    pub fn with_updates(self, updates: Arc<dyn UpdatesLink>) -> ClientBuilder {
        ClientBuilder(Client { updates, ..self.0 })
    }

    /// With the Software Update pane logging out through `log_out`.
    #[must_use]
    pub fn with_log_out(self, log_out: Arc<dyn Departure>) -> ClientBuilder {
        ClientBuilder(Client { log_out, ..self.0 })
    }

    /// With the Wi-Fi pane following `wifi`.
    #[must_use]
    pub fn with_wifi(self, wifi: Arc<dyn WifiLink>) -> ClientBuilder {
        ClientBuilder(Client { wifi, ..self.0 })
    }

    /// With the Bluetooth pane following `bluetooth`.
    #[must_use]
    pub fn with_bluetooth(self, bluetooth: Arc<dyn BluetoothLink>) -> ClientBuilder {
        ClientBuilder(Client {
            bluetooth,
            ..self.0
        })
    }

    /// With the Displays pane following `displays`.
    #[must_use]
    pub fn with_displays(self, displays: Arc<dyn DisplaysLink>) -> ClientBuilder {
        ClientBuilder(Client { displays, ..self.0 })
    }

    /// With the engine sheet working through `desk`.
    #[must_use]
    pub fn with_engines(self, desk: Arc<dyn Desk>) -> ClientBuilder {
        ClientBuilder(Client { desk, ..self.0 })
    }

    /// With the llama.cpp note over `engines`.
    #[must_use]
    pub fn with_engine_note(self, engines: Arc<dyn EnginesLink>) -> ClientBuilder {
        ClientBuilder(Client { engines, ..self.0 })
    }

    /// Looking for coding agents on `programs_path` (the value of `PATH`).
    #[must_use]
    pub fn with_programs_path(self, programs_path: String) -> ClientBuilder {
        let search = AgentSearch {
            programs_path,
            ..self.0.search.clone()
        };
        ClientBuilder(Client { search, ..self.0 })
    }

    /// Downloading coding assistants with the program at `agents_program`.
    #[must_use]
    pub fn with_agents_program(self, agents_program: Option<PathBuf>) -> ClientBuilder {
        ClientBuilder(Client {
            agents_program,
            ..self.0
        })
    }

    /// With the Wallpaper page working through `pictures`.
    #[must_use]
    pub fn with_pictures(self, pictures: Arc<dyn Pictures>) -> ClientBuilder {
        ClientBuilder(Client { pictures, ..self.0 })
    }

    /// Listing the apps' actions from the manifests in `intent_dirs`, in the order of trust.
    #[must_use]
    pub fn with_intent_dirs(self, intent_dirs: Vec<PathBuf>) -> ClientBuilder {
        let corners = Arc::new(OnCorners::new(Arc::clone(&self.0.chooser), intent_dirs));
        ClientBuilder(Client { corners, ..self.0 })
    }

    /// With the Hot Corners sheet working through `corners`.
    #[must_use]
    pub fn with_corners(self, corners: Arc<dyn CornerDesk>) -> ClientBuilder {
        ClientBuilder(Client { corners, ..self.0 })
    }

    /// Asking `chooser` for folders, files and pictures.
    #[must_use]
    pub fn with_chooser(self, chooser: Arc<dyn ChooserLink>) -> ClientBuilder {
        ClientBuilder(Client { chooser, ..self.0 })
    }

    /// With the panes that follow a daemon's live settings module looking for it in `modules`.
    #[must_use]
    pub fn with_modules(self, modules: Arc<dyn ModulesLink>) -> ClientBuilder {
        ClientBuilder(Client { modules, ..self.0 })
    }

    /// With the Accounts pane over `accounts`.
    #[must_use]
    pub fn with_accounts(self, accounts: Arc<dyn AccountsLink>) -> ClientBuilder {
        ClientBuilder(Client { accounts, ..self.0 })
    }

    /// With the Accounts pane asking the sync service through `sync`.
    #[must_use]
    pub fn with_sync(self, sync: Arc<dyn SyncLink>) -> ClientBuilder {
        ClientBuilder(Client { sync, ..self.0 })
    }

    /// With the model picker setting its rows through `picker`.
    #[must_use]
    pub fn with_picker(self, picker: Arc<dyn PickerLink>) -> ClientBuilder {
        ClientBuilder(Client { picker, ..self.0 })
    }

    /// With the Privacy page setting its rows through `privacy`.
    #[must_use]
    pub fn with_privacy(self, privacy: Arc<dyn PrivacyLink>) -> ClientBuilder {
        ClientBuilder(Client { privacy, ..self.0 })
    }

    /// With the Spaces page over `spaces`.
    #[must_use]
    pub fn with_spaces(self, spaces: Arc<dyn SpacesLink>) -> ClientBuilder {
        ClientBuilder(Client { spaces, ..self.0 })
    }

    /// With the Privacy page's portal list over `screen_sharing`.
    #[must_use]
    pub fn with_screen_sharing(self, screen_sharing: Arc<dyn ScreenSharingLink>) -> ClientBuilder {
        ClientBuilder(Client {
            screen_sharing,
            ..self.0
        })
    }

    /// With the memory consent over `memory`.
    #[must_use]
    pub fn with_memory(self, memory: Arc<dyn MemoryLink>) -> ClientBuilder {
        ClientBuilder(Client { memory, ..self.0 })
    }

    /// With the running-models list over `running`.
    #[must_use]
    pub fn with_running(self, running: Arc<dyn RunningLink>) -> ClientBuilder {
        ClientBuilder(Client { running, ..self.0 })
    }

    /// With Your computers over `computers`.
    #[must_use]
    pub fn with_computers(self, computers: Arc<dyn ComputersLink>) -> ClientBuilder {
        ClientBuilder(Client {
            computers,
            ..self.0
        })
    }

    /// With the picker's reason footer reading `reasons`.
    #[must_use]
    pub fn with_reasons(self, reasons: ReasonFeed) -> ClientBuilder {
        ClientBuilder(Client { reasons, ..self.0 })
    }
}
