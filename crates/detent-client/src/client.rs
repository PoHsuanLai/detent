//! The one door the window uses. A [`Client`] holds every service the Settings app reaches (the
//! links of `detent-system`) and the bus they are looked for on; each pane's handle comes from it,
//! and a handle is a live value to draw and the commands to give it. The window never touches a
//! link, a deadline or a follow loop.

use crate::accounts::{Accounts, Listing};
use crate::bluetooth::Bluetooth;
use crate::chooser::Chooser;
use crate::computers::Computers;
use crate::displays::Displays;
use crate::engine_install::EngineInstall;
use crate::engines::{EnginePrograms, Engines};
use crate::hot_corners::Corners;
use crate::lending::Lending;
use crate::memory_consent::MemoryConsent;
use crate::picker::{AgentSearch, Models, Picker, ReasonFeed, Reasons};
use crate::privacy::Privacy;
use crate::running::Running;
use crate::screen_sharing::ScreenSharing;
use crate::spaces::Spaces;
use crate::updates::Updates;
use crate::wallpaper::Wallpaper;
use crate::whoami::Whoami;
use crate::wifi::Wifi;
use detent_files::{ConfigHome, DataHome};
use detent_model::{Absence, AccountsPane, AddEnd, ModuleId, UnixSeconds, Work};
use detent_system::accounts::AccountsLink;
use detent_system::bluetooth::BluetoothLink;
use detent_system::bus::{BusSource, start};
use detent_system::chooser::ChooserLink;
use detent_system::computers::ComputersLink;
use detent_system::displays::DisplaysLink;
use detent_system::engine_install::Desk;
use detent_system::engines::EnginesLink;
use detent_system::hot_corners::CornerDesk;
use detent_system::lending::Lender;
use detent_system::memory_consent::MemoryLink;
use detent_system::modules::ModulesLink;
use detent_system::person::PersonLink;
use detent_system::picker::PickerLink;
use detent_system::privacy::PrivacyLink;
use detent_system::running::RunningLink;
use detent_system::screen_sharing::ScreenSharingLink;
use detent_system::spaces::SpacesLink;
use detent_system::sync::SyncLink;
use detent_system::updates::{Departure, UpdatesLink};
use detent_system::wallpaper::Pictures;
use detent_system::wifi::WifiLink;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

/// Every service the window reaches, behind one value that is cheap to clone.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) bus: BusSource,
    pub(crate) modules: Arc<dyn ModulesLink>,
    pub(crate) chooser: Arc<dyn ChooserLink>,
    pub(crate) accounts: Arc<dyn AccountsLink>,
    pub(crate) sync: Arc<dyn SyncLink>,
    pub(crate) picker: Arc<dyn PickerLink>,
    pub(crate) privacy: Arc<dyn PrivacyLink>,
    pub(crate) screen_sharing: Arc<dyn ScreenSharingLink>,
    pub(crate) spaces: Arc<dyn SpacesLink>,
    pub(crate) memory: Arc<dyn MemoryLink>,
    pub(crate) running: Arc<dyn RunningLink>,
    pub(crate) computers: Arc<dyn ComputersLink>,
    pub(crate) engines: Arc<dyn EnginesLink>,
    pub(crate) reasons: ReasonFeed,
    pub(crate) lender: Lender,
    pub(crate) updates: Arc<dyn UpdatesLink>,
    pub(crate) log_out: Arc<dyn Departure>,
    pub(crate) wifi: Arc<dyn WifiLink>,
    pub(crate) bluetooth: Arc<dyn BluetoothLink>,
    pub(crate) displays: Arc<dyn DisplaysLink>,
    pub(crate) desk: Arc<dyn Desk>,
    pub(crate) pictures: Arc<dyn Pictures>,
    pub(crate) person: Arc<dyn PersonLink>,
    pub(crate) corners: Arc<dyn CornerDesk>,
    pub(crate) search: AgentSearch,
    pub(crate) agents_program: Option<PathBuf>,
}

impl Client {
    /// Where the services are looked for.
    pub fn bus(&self) -> &BusSource {
        &self.bus
    }

    /// The program that lists and downloads coding assistants, where the launcher found it.
    pub fn agents_program(&self) -> Option<&Path> {
        self.agents_program.as_deref()
    }

    /// The settings home and the data home the client reads docket's files from.
    pub fn homes(&self) -> (&ConfigHome, &DataHome) {
        (&self.search.config_home, &self.search.data_home)
    }

    /// Asks the bus to start the services `modules` are served by, once, as a page is shown.
    /// Nothing is started for a page that was not opened.
    pub async fn start(&self, modules: &[ModuleId]) {
        start(&self.bus, modules).await;
    }

    /// The accounts the modules list now, read once; never starts a daemon.
    pub fn read_accounts(&self) -> Work<'_, Result<AccountsPane, Absence>> {
        self.accounts.read()
    }

    /// Asks accountd for the add sheet and waits for it to end.
    pub async fn add_account(&self) -> AddEnd {
        self.accounts.add_account().await
    }

    /// The time "last checked" and "allowed at" count from.
    pub fn now(&self) -> UnixSeconds {
        self.updates.now()
    }

    /// The desktop's file chooser.
    pub fn chooser(&self) -> Chooser {
        Chooser::over(Arc::clone(&self.chooser))
    }

    /// The Wi-Fi pane.
    pub fn wifi(&self) -> Wifi {
        Wifi::over(Arc::clone(&self.wifi))
    }

    /// The Bluetooth pane.
    pub fn bluetooth(&self) -> Bluetooth {
        Bluetooth::over(Arc::clone(&self.bluetooth))
    }

    /// The Displays pane.
    pub fn displays(&self) -> Displays {
        Displays::over(Arc::clone(&self.displays))
    }

    /// The Software Update pane.
    pub fn updates(&self) -> Updates {
        Updates::over(Arc::clone(&self.updates), Arc::clone(&self.log_out))
    }

    /// The Accounts pane, with the held deletions and the datasets of its accounts.
    pub fn accounts(&self) -> Accounts {
        Accounts::over(
            self.bus.clone(),
            Arc::clone(&self.modules),
            Arc::clone(&self.accounts),
            Arc::clone(&self.sync),
        )
    }

    /// The accounts the modules list, for a page outside the Accounts pane.
    pub fn listing(&self) -> Listing {
        Listing::over(Arc::clone(&self.modules))
    }

    /// The model picker.
    pub fn picker(&self) -> Picker {
        Picker::over(
            Arc::clone(&self.modules),
            Arc::clone(&self.picker),
            Arc::clone(&self.accounts),
            self.search.clone(),
        )
    }

    /// The models inferd lists, where a place offers them to choose from.
    pub fn models(&self) -> Models {
        Models::over(Arc::clone(&self.modules))
    }

    /// The reasons the picker's footer says; handed over once, to the first caller.
    pub fn reasons(&self) -> Reasons {
        self.reasons.take()
    }

    /// The engines Local can run models with, and the download of one.
    pub fn engine_install(&self) -> EngineInstall {
        EngineInstall::over(Arc::clone(&self.desk))
    }

    /// The llama.cpp note under "Models on this computer".
    pub fn engines(&self) -> Engines {
        Engines::over(self.bus.clone(), Arc::clone(&self.engines))
    }

    /// The rows that name the program of an engine.
    pub fn engine_programs(&self) -> EnginePrograms {
        EnginePrograms::over(Arc::clone(&self.modules), Arc::clone(&self.picker))
    }

    /// The models inferd runs itself.
    pub fn running(&self) -> Running {
        Running::over(self.bus.clone(), Arc::clone(&self.running))
    }

    /// Your computers.
    pub fn computers(&self) -> Computers {
        Computers::over(self.bus.clone(), Arc::clone(&self.computers))
    }

    /// The Privacy pane.
    pub fn privacy(&self) -> Privacy {
        Privacy::over(Arc::clone(&self.modules), Arc::clone(&self.privacy))
    }

    /// The apps that may share the screen without asking again.
    pub fn screen_sharing(&self) -> ScreenSharing {
        ScreenSharing::over(Arc::clone(&self.screen_sharing))
    }

    /// The desktop-wide Spaces.
    pub fn spaces(&self) -> Spaces {
        Spaces::over(Arc::clone(&self.spaces))
    }

    /// The looser memory settings.
    pub fn memory_consent(&self) -> MemoryConsent {
        MemoryConsent::over(Arc::clone(&self.memory))
    }

    /// The switch that lends this computer's models to the person's other computers.
    pub fn lending(&self) -> Lending {
        Lending::over(self.lender.clone())
    }

    /// The pictures behind the Wallpaper page.
    pub fn wallpaper(&self) -> Wallpaper {
        Wallpaper::over(Arc::clone(&self.pictures))
    }

    /// The person at the top of the sidebar.
    pub fn whoami(&self) -> Whoami {
        Whoami::over(Arc::clone(&self.person))
    }

    /// The Hot Corners sheet's desk.
    pub fn corners(&self) -> Corners {
        Corners::over(Arc::clone(&self.corners))
    }
}
