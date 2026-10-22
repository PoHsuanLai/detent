//! The groups the list is drawn in, and the actions of each.

use chordkit::{SpaceNumber, StandardAction as A};
use keycap_client::handler;

/// One titled group of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutGroup {
    /// Opening the launcher.
    Launcher,
    /// Switching between, hiding and closing apps and windows.
    Windows,
    /// Tiling, filling and centring the window.
    Tiling,
    /// Moving between Spaces.
    Spaces,
    /// Capturing the screen.
    Screenshots,
    /// Input languages and the emoji picker.
    Input,
    /// Editing and styling text.
    Text,
    /// Documents, panes and help.
    Documents,
    /// Locking and leaving the session.
    System,
}

const LAUNCHER: &[A] = &[A::Launcher];
const WINDOWS: &[A] = &[
    A::NextApp,
    A::PreviousApp,
    A::NextWindow,
    A::AppWindows,
    A::Hide,
    A::HideOthers,
    A::Minimize,
    A::MinimizeAll,
    A::Close,
    A::CloseAll,
    A::Quit,
    A::FullScreen,
    A::ForceQuit,
];
const TILING: &[A] = &[
    A::TileLeft,
    A::TileRight,
    A::TileTop,
    A::TileBottom,
    A::TileTopLeft,
    A::TileTopRight,
    A::TileBottomLeft,
    A::TileBottomRight,
    A::Fill,
    A::Center,
    A::ReturnToPreviousSize,
];
const SPACES: &[A] = &[A::MissionControl, A::SpaceLeft, A::SpaceRight];
const SCREENSHOTS: &[A] = &[
    A::ScreenshotScreen,
    A::ScreenshotSelection,
    A::ScreenshotTools,
];
const INPUT: &[A] = &[A::NextInputSource, A::EmojiAndSymbols];
const TEXT: &[A] = &[
    A::SelectAll,
    A::Copy,
    A::Cut,
    A::Paste,
    A::PasteAndMatchStyle,
    A::Undo,
    A::Redo,
    A::Find,
    A::FindNext,
    A::FindPrevious,
    A::UseSelectionForFind,
    A::JumpToSelection,
    A::Bold,
    A::Italic,
    A::Underline,
    A::ShowFonts,
    A::ShowColors,
    A::Bigger,
    A::Smaller,
    A::ActualSize,
];
const DOCUMENTS: &[A] = &[
    A::New,
    A::Open,
    A::Save,
    A::SaveAs,
    A::Print,
    A::PageSetup,
    A::ToggleToolbar,
    A::ToggleSidebar,
    A::ToggleDock,
    A::QuickLook,
    A::Reveal,
    A::NewFolder,
    A::MoveToTrash,
    A::EmptyTrash,
    A::Settings,
    A::Help,
    A::Cancel,
];
const SYSTEM: &[A] = &[A::LockScreen, A::LogOut];
/// The window actions of an app's own menus (as on a Mac: the app menu's Hide and Quit, the
/// Window menu's Minimize, the File menu's Close): every app handles these itself, though they
/// are listed with the desktop's app switching.
const APP_MENU: &[A] = &[
    A::Hide,
    A::HideOthers,
    A::Minimize,
    A::MinimizeAll,
    A::Close,
    A::CloseAll,
    A::Quit,
];

impl ShortcutGroup {
    /// Every group, in the order the list draws them.
    pub const ALL: [ShortcutGroup; 9] = [
        ShortcutGroup::Launcher,
        ShortcutGroup::Windows,
        ShortcutGroup::Tiling,
        ShortcutGroup::Spaces,
        ShortcutGroup::Screenshots,
        ShortcutGroup::Input,
        ShortcutGroup::Text,
        ShortcutGroup::Documents,
        ShortcutGroup::System,
    ];

    /// The group's title.
    pub fn title(self) -> &'static str {
        match self {
            ShortcutGroup::Launcher => "Launcher",
            ShortcutGroup::Windows => "Apps and windows",
            ShortcutGroup::Tiling => "Window tiling",
            ShortcutGroup::Spaces => "Spaces",
            ShortcutGroup::Screenshots => "Screenshots",
            ShortcutGroup::Input => "Input",
            ShortcutGroup::Text => "Text",
            ShortcutGroup::Documents => "Documents and views",
            ShortcutGroup::System => "System",
        }
    }

    /// Whether the group is the desktop's own (it lists only what the desktop handles) rather
    /// than an app's (every app handles its own).
    pub fn is_desktop(self) -> bool {
        !matches!(self, ShortcutGroup::Text | ShortcutGroup::Documents)
    }

    /// The actions of the group, in the order they are listed: a desktop group lists only the
    /// actions our desktop handles, so no row is a shortcut that does nothing.
    pub fn actions(self) -> Vec<A> {
        self.listed(|action| handler(action).is_some())
    }

    /// The actions of the group, handled or not.
    pub fn all_actions(self) -> Vec<A> {
        self.listed(|_| true)
    }

    /// Whether `action` in this group is the desktop's to handle, not each app's.
    pub fn is_desktop_action(self, action: A) -> bool {
        self.is_desktop() && !APP_MENU.contains(&action)
    }

    /// The group's actions with those `handled` keeps; an app's action is kept whatever
    /// `handled` says.
    pub fn listed(self, handled: impl Fn(A) -> bool) -> Vec<A> {
        let fixed = match self {
            ShortcutGroup::Launcher => LAUNCHER,
            ShortcutGroup::Windows => WINDOWS,
            ShortcutGroup::Tiling => TILING,
            ShortcutGroup::Spaces => SPACES,
            ShortcutGroup::Screenshots => SCREENSHOTS,
            ShortcutGroup::Input => INPUT,
            ShortcutGroup::Text => TEXT,
            ShortcutGroup::Documents => DOCUMENTS,
            ShortcutGroup::System => SYSTEM,
        };
        let numbered = (1..=9)
            .filter_map(SpaceNumber::new)
            .map(A::SwitchToSpace)
            .filter(|_| self == ShortcutGroup::Spaces);
        fixed
            .iter()
            .copied()
            .chain(numbered)
            .filter(|action| !self.is_desktop_action(*action) || handled(*action))
            .collect()
    }
}
