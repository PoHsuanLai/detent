//! The tile beside each page in the sidebar: one glyph on one ground, as System Settings has them.
//! The model names the ground of each kind of page; the colours of the grounds are this one table.

use detent_model::{Ground, PageKind};
use ds::prelude::{Icon, TileFace};
use ds::style::tokens::hex::Hex;

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
        PageKind::Privacy => Icon::Hand,
        PageKind::Power => Icon::Power,
        PageKind::Apps => Icon::Grid,
        PageKind::Intelligence => Icon::Sparkles,
        PageKind::Viewer => Icon::Image,
        PageKind::SoftwareUpdate => Icon::Download,
        PageKind::Wallpaper => Icon::Image,
        PageKind::AssistantModel => Icon::Switches,
        PageKind::Wifi => Icon::Wifi,
        PageKind::Bluetooth => Icon::Bluetooth,
        PageKind::Displays => Icon::Monitor,
        PageKind::Extensions => Icon::Puzzle,
        PageKind::Program => Icon::Window,
    }
}

/// The colour of `ground`.
pub const fn hex_of(ground: Ground) -> Hex {
    match ground {
        Ground::Blue => Hex([0x0a, 0x84, 0xff]),
        Ground::Red => Hex([0xff, 0x3b, 0x30]),
        Ground::Graphite => Hex([0x3a, 0x3a, 0x3c]),
        Ground::Cyan => Hex([0x32, 0xad, 0xe6]),
        Ground::Green => Hex([0x34, 0xc7, 0x59]),
        Ground::Grey => Hex([0x8e, 0x8e, 0x93]),
        Ground::Indigo => Hex([0x5e, 0x5c, 0xe6]),
    }
}

/// `icon` in white on `ground`.
pub fn tile_on(icon: Icon, ground: Ground) -> TileFace {
    TileFace::Glyph(icon, hex_of(ground))
}

/// The tile of a page of `kind`.
pub fn tile_of(kind: PageKind) -> TileFace {
    tile_on(icon_of(kind), kind.ground())
}

#[cfg(test)]
mod tests {
    use super::{hex_of, icon_of};
    use detent_model::{Ground, PageKind};

    #[test]
    fn intelligence_and_the_assistant_model_have_different_glyphs() {
        assert_ne!(
            icon_of(PageKind::Intelligence),
            icon_of(PageKind::AssistantModel)
        );
    }

    #[test]
    fn the_grounds_are_all_different() {
        let grounds = [
            Ground::Blue,
            Ground::Red,
            Ground::Graphite,
            Ground::Cyan,
            Ground::Green,
            Ground::Grey,
            Ground::Indigo,
        ];
        for (at, one) in grounds.iter().enumerate() {
            for other in &grounds[at + 1..] {
                assert_ne!(hex_of(*one), hex_of(*other), "{one:?} and {other:?}");
            }
        }
    }
}
