//! The glyph beside each page in the sidebar.

use detent_model::{is_assistant_model, is_software_update};
use ds::prelude::Icon;
use ds_settings::schema::Page;

/// The sidebar glyph of `page`.
pub fn icon_of(page: &Page) -> Icon {
    match page {
        Page::Appearance => Icon::SunMoon,
        Page::Dock => Icon::Panel,
        Page::MouseAndGestures => Icon::Mouse,
        Page::KeyboardAndShortcuts => Icon::Keyboard,
        Page::Notifications => Icon::Bell,
        Page::Spaces => Icon::Group,
        Page::Accounts => Icon::Key,
        Page::Power => Icon::Power,
        Page::Apps => Icon::Grid,
        Page::Intelligence => Icon::Sparkles,
        Page::Viewer => Icon::Image,
        Page::App(_) if is_software_update(page) => Icon::Download,
        Page::App(_) if is_assistant_model(page) => Icon::Sparkles,
        Page::App(_) => Icon::Window,
    }
}
