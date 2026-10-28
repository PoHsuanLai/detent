//! The strip at the top of the Dock page: a few tiles on a pill, each as big as the Size slider
//! says, so the slider's effect is seen at once. It reads the value; it sets nothing.

use crate::copy::dock::DOCK_PREVIEW;
use crate::files::{Files, value_of};
use crate::icons::DOCK_SAMPLE;
use detent_model::PageModel;
use detent_model::external::toml;
use dioxus::prelude::*;
use ds::prelude::IconTile;

/// The key whose value sizes the tiles.
const TILE_SIZE_KEY: &str = "dock.tile_size_px";

/// The side of a tile when the page holds no size, in pixels (the dock's resting size).
const DEFAULT_PX: i64 = 48;

/// The smallest and the largest side drawn, so a stray value cannot make a strip of nothing or
/// one taller than the window.
const SIDES: (i64, i64) = (8, 128);

/// The side of a tile for `value`: a whole or a fractional number, kept within [`SIDES`].
fn tile_px(value: &toml::Value) -> i64 {
    let side = value
        .as_integer()
        // A fraction of a pixel makes no difference to a picture this size.
        .or_else(|| value.as_float().map(|float| float.round() as i64))
        .unwrap_or(DEFAULT_PX);
    side.clamp(SIDES.0, SIDES.1)
}

/// The tiles on a pill at the page's current `dock.tile_size_px`.
#[component]
pub(crate) fn DockPreview(page: PageModel, files: Signal<Files>) -> Element {
    let size = page
        .sections
        .iter()
        .flat_map(|section| &section.keys)
        .find(|entry| entry.spec.path.0 == TILE_SIZE_KEY)
        .map_or(DEFAULT_PX, |entry| tile_px(&value_of(&files(), entry)));
    rsx! {
        div {
            class: "dock-preview",
            role: "img",
            "aria-label": DOCK_PREVIEW,
            "data-dock-preview": "{size}",
            div {
                class: "dock-preview-pill",
                style: "--row-icon-tile:{size}px",
                for (index , (icon , hex)) in DOCK_SAMPLE.into_iter().enumerate() {
                    span { key: "{index}", "data-dock-tile": "{index}", IconTile { icon, colour: hex } }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_PX, tile_px};
    use detent_model::external::toml::Value;

    #[test]
    fn the_side_follows_the_number_within_limits() {
        // (value, side)
        let cases = [
            (Value::Integer(64), 64),
            (Value::Float(40.4), 40),
            (Value::Integer(0), 8),
            (Value::Integer(900), 128),
            (Value::String("big".to_owned()), DEFAULT_PX),
        ];
        for (value, side) in cases {
            assert_eq!(tile_px(&value), side, "{value:?}");
        }
    }
}
