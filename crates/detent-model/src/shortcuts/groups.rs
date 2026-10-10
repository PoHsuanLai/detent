//! The groups the list is drawn in, and the actions of each.

use chordkit::{SpaceNumber, StandardAction as A};

/// One titled group of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutGroup {
    /// Opening the launcher.
    Launcher,
    /// Switching between, hiding and closing apps and windows.
    Windows,
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
    A::Settings,
    A::Help,
    A::Cancel,
];
const SYSTEM: &[A] = &[A::LockScreen, A::LogOut];

impl ShortcutGroup {
    /// Every group, in the order the list draws them.
    pub const ALL: [ShortcutGroup; 8] = [
        ShortcutGroup::Launcher,
        ShortcutGroup::Windows,
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
            ShortcutGroup::Spaces => "Spaces",
            ShortcutGroup::Screenshots => "Screenshots",
            ShortcutGroup::Input => "Input",
            ShortcutGroup::Text => "Text",
            ShortcutGroup::Documents => "Documents and views",
            ShortcutGroup::System => "System",
        }
    }

    /// The actions of the group, in the order they are listed.
    pub fn actions(self) -> Vec<A> {
        let fixed = match self {
            ShortcutGroup::Launcher => LAUNCHER,
            ShortcutGroup::Windows => WINDOWS,
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
        fixed.iter().copied().chain(numbered).collect()
    }
}
