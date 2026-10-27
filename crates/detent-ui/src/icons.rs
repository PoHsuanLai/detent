//! The glyph beside each page in the sidebar.

use detent_model::PageKind;
use ds::prelude::Icon;

/// The sidebar glyph of a page of `kind`.
pub fn icon_of(kind: PageKind) -> Icon {
    match kind {
        PageKind::Appearance => Icon::SunMoon,
        PageKind::Dock => Icon::Panel,
        PageKind::MouseAndGestures => Icon::Mouse,
        PageKind::KeyboardAndShortcuts => Icon::Keyboard,
        PageKind::Notifications => Icon::Bell,
        PageKind::Spaces => Icon::Group,
        PageKind::Accounts => Icon::Key,
        PageKind::Privacy => Icon::Lock,
        PageKind::Power => Icon::Power,
        PageKind::Apps => Icon::Grid,
        PageKind::Intelligence => Icon::Sparkles,
        PageKind::Viewer => Icon::Image,
        PageKind::SoftwareUpdate => Icon::Download,
        PageKind::Wallpaper => Icon::Image,
        PageKind::AssistantModel => Icon::Sparkles,
        PageKind::Wifi => Icon::Wifi,
        PageKind::Bluetooth => Icon::Bluetooth,
        PageKind::Displays => Icon::Monitor,
        PageKind::Extensions => Icon::Puzzle,
        PageKind::Program => Icon::Window,
    }
}
