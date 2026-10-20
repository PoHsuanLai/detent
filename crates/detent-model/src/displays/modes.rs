//! The modes a display lists: the pixel sizes it can draw, and which mode a size means.

use super::model::{DisplayMode, ModePreference, PixelSize};

/// The distinct pixel sizes `modes` list, the largest first.
pub fn mode_sizes(modes: &[DisplayMode]) -> Vec<PixelSize> {
    let mut sizes: Vec<PixelSize> = modes.iter().map(|mode| mode.size).collect();
    sizes.sort_by_key(|size| {
        std::cmp::Reverse((u64::from(size.width) * u64::from(size.height), *size))
    });
    sizes.dedup();
    sizes
}

/// The size of the mode the display prefers, its native one.
pub fn native_size(modes: &[DisplayMode]) -> Option<PixelSize> {
    modes
        .iter()
        .find(|mode| mode.preferred == ModePreference::Yes)
        .map(|mode| mode.size)
}

/// The mode a change to `size` runs: the one at `size` with the refresh rate the display runs now,
/// else the fastest at `size`. None when the display lists no mode of that size.
pub fn mode_at(
    modes: &[DisplayMode],
    current: Option<DisplayMode>,
    size: PixelSize,
) -> Option<DisplayMode> {
    let at_size = || modes.iter().copied().filter(|mode| mode.size == size);
    current
        .and_then(|now| at_size().find(|mode| mode.refresh == now.refresh))
        .or_else(|| at_size().max_by_key(|mode| mode.refresh))
}
