//! One row per kind of page: everything the model says about a kind that is not its identity.
//! Adding a page is one row here and one row in the window's look table.

use crate::accounts::ModuleId;
use crate::assistant_model::ASSISTANT_MODEL_SLUG;
use crate::bluetooth::BLUETOOTH_SLUG;
use crate::displays::DISPLAYS_SLUG;
use crate::extensions::EXTENSIONS_SLUG;
use crate::page_kind::PageKind;
use crate::sidebar_group::SidebarGroup;
use crate::updates::SOFTWARE_UPDATE_SLUG;
use crate::wallpaper::WALLPAPER_SLUG;
use crate::wifi::WIFI_SLUG;

/// A block of a page that is not in any file: the window mounts the pane of this name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivePane {
    /// accountd's Accounts pane.
    Accounts,
    /// The programs and the powers the compositor lists for them.
    Privacy,
    /// The apps the screen-sharing portal lets share the screen without asking again.
    ScreenSharing,
    /// The desktop-wide Spaces accountd keeps.
    Spaces,
    /// The desktop's shortcuts, grouped, each with its chord and who set it.
    Shortcuts,
    /// The update service's page.
    SoftwareUpdate,
    /// The assistant's model: inferd's picker, reduced to the one row the assistant is routed by.
    AssistantModel,
    /// The Wi-Fi switch, the network this computer is on and the ones in range.
    Wifi,
    /// The Bluetooth switch and the devices.
    Bluetooth,
    /// Each display's resolution, refresh rate, rotation and place.
    Displays,
    /// The installed Extensions, grouped by what they provide.
    Extensions,
    /// Where the built-in assistant may run, and the assistants that sign themselves in.
    Assistants,
    /// The local runtimes accountd lists (Ollama, llama.cpp, LM Studio), each linked to its account.
    LocalRuntimes,
    /// inferd's live model picker.
    ModelPicker,
}

/// How a page lays out its sections and its live panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Body {
    /// Its sections as the schemas gave them.
    Sections,
    /// The panes, then the sections.
    Before(&'static [LivePane]),
    /// The sections, then the panes.
    After(&'static [LivePane]),
    /// The pane and nothing else.
    Only(LivePane),
    /// The sections in design/22's order with the assistant panes slotted in.
    Intelligence,
    /// The sections, with the style notes after the one that holds the styles switch.
    Styles,
}

/// What the model says about one kind of page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTraits {
    /// The page's stable name, or none for a program's page, which is named by its id.
    pub slug: Option<&'static str>,
    /// The sidebar group the page sits in.
    pub group: SidebarGroup,
    /// The page's place inside its group.
    pub place: u8,
    /// Whether the page opens with a hero: its tile, its name and one sentence.
    pub hero: bool,
    /// The daemons the page reads while it is shown, each once. Privacy reads none: the
    /// compositor publishes the rows, and Settings looks for it and never starts it.
    pub modules: &'static [ModuleId],
    /// What the page draws.
    pub body: Body,
}

impl PageTraits {
    /// The group and the place inside it: the order the sidebar draws.
    pub fn slot(&self) -> (SidebarGroup, u8) {
        (self.group, self.place)
    }

    const fn row(slug: Option<&'static str>, group: SidebarGroup, place: u8) -> PageTraits {
        PageTraits {
            slug,
            group,
            place,
            hero: false,
            modules: &[],
            body: Body::Sections,
        }
    }

    const fn with_hero(mut self) -> PageTraits {
        self.hero = true;
        self
    }

    const fn reading(mut self, modules: &'static [ModuleId]) -> PageTraits {
        self.modules = modules;
        self
    }

    const fn drawing(mut self, body: Body) -> PageTraits {
        self.body = body;
        self
    }
}

/// The three daemons the Accounts pane lists accounts from, in the order it lists them.
const ACCOUNT_MODULES: &[ModuleId] = &[ModuleId::Accountd, ModuleId::Inferd, ModuleId::Syncd];

/// The one daemon the Assistant Model page reads: inferd's picker.
const INFERD: &[ModuleId] = &[ModuleId::Inferd];

/// accountd, which keeps the Spaces registry.
const ACCOUNTD: &[ModuleId] = &[ModuleId::Accountd];

impl PageKind {
    /// The page's row. The match names every kind, so a new page cannot be added without
    /// deciding each of its traits; Intelligence also reads accountd and syncd for the accounts
    /// the local runtimes and the cloud places read.
    pub fn traits(self) -> PageTraits {
        use Body::{After, Before, Intelligence, Only, Styles};
        use LivePane as P;
        use SidebarGroup::{Accounts, Alerts, Connection, Desktop, Input, Look, Programs, System};
        let row = PageTraits::row;
        match self {
            PageKind::Wifi => row(Some(WIFI_SLUG), Connection, 0).drawing(Only(P::Wifi)),
            PageKind::Bluetooth => {
                row(Some(BLUETOOTH_SLUG), Connection, 1).drawing(Only(P::Bluetooth))
            }
            PageKind::Notifications => row(Some("notifications"), Alerts, 0),
            PageKind::Appearance => row(Some("appearance"), Look, 0).drawing(Styles),
            PageKind::Intelligence => row(Some("intelligence"), Look, 1)
                .with_hero()
                .reading(ACCOUNT_MODULES)
                .drawing(Intelligence),
            PageKind::AssistantModel => row(Some(ASSISTANT_MODEL_SLUG), Look, 2)
                .reading(INFERD)
                .drawing(Only(P::AssistantModel)),
            PageKind::Extensions => {
                row(Some(EXTENSIONS_SLUG), Look, 3).drawing(Only(P::Extensions))
            }
            PageKind::Privacy => row(Some("privacy"), Look, 4)
                .with_hero()
                .drawing(Before(&[P::Privacy, P::ScreenSharing])),
            PageKind::Dock => row(Some("dock"), Desktop, 0),
            PageKind::Displays => row(Some(DISPLAYS_SLUG), Desktop, 1).drawing(Only(P::Displays)),
            PageKind::Wallpaper => row(Some(WALLPAPER_SLUG), Desktop, 2),
            PageKind::Spaces => row(Some("spaces"), Desktop, 3)
                .reading(ACCOUNTD)
                .drawing(Before(&[P::Spaces])),
            PageKind::Power => row(Some("power"), System, 0),
            PageKind::SoftwareUpdate => {
                row(Some(SOFTWARE_UPDATE_SLUG), System, 1).drawing(Only(P::SoftwareUpdate))
            }
            PageKind::Accounts => row(Some("accounts"), Accounts, 0)
                .with_hero()
                .reading(ACCOUNT_MODULES)
                .drawing(Before(&[P::Accounts])),
            PageKind::KeyboardAndShortcuts => {
                row(Some("keyboard_and_shortcuts"), Input, 0).drawing(After(&[P::Shortcuts]))
            }
            PageKind::MouseAndGestures => row(Some("mouse_and_gestures"), Input, 1),
            PageKind::Apps => row(Some("apps"), Programs, 0),
            PageKind::Viewer => row(Some("viewer"), Programs, 0),
            PageKind::Program => row(None, Programs, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Body, LivePane};
    use crate::page_kind::PageKind;

    #[test]
    fn only_the_overview_pages_open_with_a_hero() {
        for kind in PageKind::ALL {
            let overview = matches!(
                kind,
                PageKind::Privacy | PageKind::Accounts | PageKind::Intelligence
            );
            assert_eq!(kind.traits().hero, overview, "{kind:?}");
        }
    }

    #[test]
    fn a_live_page_draws_its_pane_and_a_file_page_draws_sections() {
        assert_eq!(PageKind::Wifi.traits().body, Body::Only(LivePane::Wifi));
        assert_eq!(PageKind::Dock.traits().body, Body::Sections);
    }
}
