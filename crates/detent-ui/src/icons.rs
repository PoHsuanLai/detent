//! The tile beside each page in the sidebar: one glyph on one colour, as System Settings has them.
//! The model names the page; the glyph and the colour of each kind of page are this one table.

use detent_model::PageKind;
use ds::prelude::{Icon, TileFace};
use ds::style::tokens::hex::Hex;

const BLUE: Hex = Hex([0x0a, 0x84, 0xff]);
const RED: Hex = Hex([0xff, 0x3b, 0x30]);
const GRAPHITE: Hex = Hex([0x3a, 0x3a, 0x3c]);
const CYAN: Hex = Hex([0x32, 0xad, 0xe6]);
const GREEN: Hex = Hex([0x34, 0xc7, 0x59]);
/// The plain system pages: updates, input devices, extensions, programs.
pub const GREY: Hex = Hex([0x8e, 0x8e, 0x93]);
/// The assistant and Spaces.
pub const INDIGO: Hex = Hex([0x5e, 0x5c, 0xe6]);

/// The sidebar glyph and the colour of a page of `kind`.
pub fn look_of(kind: PageKind) -> (Icon, Hex) {
    match kind {
        PageKind::Appearance => (Icon::SunMoon, GRAPHITE),
        PageKind::Dock => (Icon::Panel, GRAPHITE),
        PageKind::MouseAndGestures => (Icon::Mouse, GREY),
        PageKind::KeyboardAndShortcuts => (Icon::Keyboard, GREY),
        PageKind::Notifications => (Icon::Bell, RED),
        PageKind::Spaces => (Icon::Group, INDIGO),
        PageKind::Accounts => (Icon::Key, BLUE),
        PageKind::Privacy => (Icon::Hand, BLUE),
        PageKind::Power => (Icon::Power, GREEN),
        PageKind::Apps => (Icon::Grid, GREY),
        PageKind::Intelligence => (Icon::Sparkles, INDIGO),
        PageKind::Viewer => (Icon::Image, GREY),
        PageKind::SoftwareUpdate => (Icon::Download, GREY),
        PageKind::Wallpaper => (Icon::Image, CYAN),
        PageKind::AssistantModel => (Icon::Switches, INDIGO),
        PageKind::Wifi => (Icon::Wifi, BLUE),
        PageKind::Bluetooth => (Icon::Bluetooth, BLUE),
        PageKind::Displays => (Icon::Monitor, BLUE),
        PageKind::Extensions => (Icon::Puzzle, GREY),
        PageKind::Program => (Icon::Window, GREY),
    }
}

/// `icon` in white on `hex`.
pub fn tile_on(icon: Icon, hex: Hex) -> TileFace {
    TileFace::Glyph(icon, hex)
}

/// The tile of a page of `kind`.
pub fn tile_of(kind: PageKind) -> TileFace {
    let (icon, hex) = look_of(kind);
    tile_on(icon, hex)
}

#[cfg(test)]
mod tests {
    use super::{BLUE, CYAN, GRAPHITE, GREEN, GREY, INDIGO, RED, look_of};
    use detent_model::PageKind;

    #[test]
    fn intelligence_and_the_assistant_model_have_different_glyphs() {
        assert_ne!(
            look_of(PageKind::Intelligence).0,
            look_of(PageKind::AssistantModel).0
        );
    }

    #[test]
    fn the_colours_are_all_different() {
        let colours = [BLUE, RED, GRAPHITE, CYAN, GREEN, GREY, INDIGO];
        for (at, one) in colours.iter().enumerate() {
            for other in &colours[at + 1..] {
                assert_ne!(one, other);
            }
        }
    }
}
